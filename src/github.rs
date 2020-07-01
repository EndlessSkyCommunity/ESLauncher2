use anyhow::Result;
use chrono::Utc;
use progress_streams::ProgressReader;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::fs::File;
use std::io::{copy, BufReader};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Deserialize, Debug)]
pub struct Repo {
    pub(crate) id: u32,
}

pub trait Artifact {
    fn name(&self) -> &str;
}

#[derive(Deserialize, Debug)]
pub struct PR {
    pub head: PRHead,
}

#[derive(Deserialize, Debug)]
pub struct PRHead {
    #[serde(alias = "ref")]
    pub branch: String,
    pub repo: Repo,
    pub sha: String,
}

pub fn get_pr(id: u16) -> Result<PR> {
    make_request(&format!(
        "https://api.github.com/repos/endless-sky/endless-sky/pulls/{}",
        id
    ))
}

#[derive(Deserialize, Debug)]
pub struct UnblockedArtifact {
    pub url: String,
}

pub fn unblock_artifact_download(artifact_id: u32) -> Result<UnblockedArtifact> {
    make_request(&format!(
        "https://endlesssky.mcofficer.me/actions-artifacts/artifact/{}",
        artifact_id
    ))
}

#[derive(Deserialize, Debug)]
struct Workflows {
    workflows: Vec<Workflow>,
}

#[derive(Deserialize, Debug)]
pub struct Workflow {
    name: String,
    pub(crate) id: u32,
}

pub fn get_cd_workflow() -> Result<Workflow> {
    let workflows: Workflows =
        make_request("https://api.github.com/repos/endless-sky/endless-sky/actions/workflows")?;
    for workflow in workflows.workflows {
        if workflow.name.eq("CD") {
            info!("Found workflow with name 'CD', id {}", workflow.id);
            return Ok(workflow);
        }
    }
    Err(anyhow!("Failed to find artifact with name 'CD'",))
}

#[derive(Deserialize, Debug)]
pub struct WorkflowRuns {
    workflow_runs: Vec<WorkflowRun>,
}

#[derive(Deserialize, Debug)]
pub struct WorkflowRun {
    pub(crate) id: u32,
    run_number: u32,
    head_repository: Option<Repo>,
}

pub fn get_latest_workflow_run(
    workflow_id: u32,
    branch: String,
    head_repo_id: u32,
) -> Result<WorkflowRun> {
    let mut pages: Vec<WorkflowRuns> = make_paginated_request(&format!(
        "https://api.github.com/repos/endless-sky/endless-sky/actions/workflows/{}/runs?branch={}",
        workflow_id, branch
    ))?;

    let runs: Vec<WorkflowRun> = pages
        .drain(..)
        .map(|runs| runs.workflow_runs)
        .flatten()
        .collect();

    info!("Got {} runs for workflow {}", runs.len(), workflow_id);
    runs
        .into_iter()
        .filter(|run| {
            run.head_repository.is_some()
                && run.head_repository.as_ref().unwrap().id.eq(&head_repo_id)
        })
        .max_by_key(|run| run.run_number)
        .ok_or_else(|| anyhow!("Found no suitable workflow runs! This can happen if the PR doesn't have the changes that produce usable builds."))
}
#[derive(Deserialize, Debug)]
struct WorkflowRunArtifacts {
    artifacts: Vec<WorkflowRunArtifact>,
}

#[derive(Deserialize, Debug)]
pub struct WorkflowRunArtifact {
    pub id: u32,
    name: String,
}

impl Artifact for WorkflowRunArtifact {
    fn name(&self) -> &str {
        &self.name
    }
}

pub fn get_workflow_run_artifacts(run_id: u32) -> Result<Vec<WorkflowRunArtifact>> {
    let artifacts: WorkflowRunArtifacts = make_request(&format!(
        "https://api.github.com/repos/endless-sky/endless-sky/actions/runs/{}/artifacts",
        run_id
    ))?;
    info!(
        "Got {} artifacts for workflow run {}",
        artifacts.artifacts.len(),
        run_id
    );
    Ok(artifacts.artifacts)
}
#[derive(Deserialize, Debug)]
pub struct GitRef {
    pub object: GitObject,
}

#[derive(Deserialize, Debug)]
pub struct GitObject {
    pub sha: String,
}

pub fn get_git_ref(name: &str) -> Result<GitRef> {
    make_request(&format!(
        "https://api.github.com/repos/endless-sky/endless-sky/git/ref/{}",
        name
    ))
}

#[derive(Deserialize, Debug)]
pub struct Release {
    pub id: i64,
    pub tag_name: String,
    assets_url: String,
}

pub fn get_release_by_tag(tag: &str) -> Result<Release> {
    make_request(&format!(
        "https://api.github.com/repos/endless-sky/endless-sky/releases/tags/{}",
        tag
    ))
}

pub fn get_latest_release(repo_slug: &str) -> Result<Release> {
    make_request(&format!(
        "https://api.github.com/repos/{}/releases/latest",
        repo_slug
    ))
}

#[derive(Deserialize, Debug)]
struct ReleaseAssets(Vec<ReleaseAsset>);

#[derive(Deserialize, Debug)]
pub struct ReleaseAsset {
    pub id: i64,
    name: String,
    pub browser_download_url: String,
}

impl Artifact for ReleaseAsset {
    fn name(&self) -> &str {
        &self.name
    }
}

pub fn get_release_assets(release_id: i64) -> Result<Vec<ReleaseAsset>> {
    let assets: ReleaseAssets = make_request(&format!(
        "https://api.github.com/repos/endless-sky/endless-sky/releases/{}/assets",
        release_id
    ))?;
    info!("Got {} assets for release {}", assets.0.len(), release_id);
    Ok(assets.0)
}

fn make_request<T: DeserializeOwned>(url: &str) -> Result<T> {
    let res = ureq::get(url).set("User-Agent", "ESLauncher2").call();
    check_ratelimit(&res);
    Ok(res.into_json_deserialize()?)
}

fn make_paginated_request<T: DeserializeOwned>(url: &str) -> Result<Vec<T>> {
    let mut next_url = Some(url.to_string());
    let mut results = vec![];

    while next_url.is_some() {
        let url = next_url.clone().unwrap();
        let res = ureq::get(&url).set("User-Agent", "ESLauncher2").call();
        check_ratelimit(&res);

        if let Some(link_header) = res.header("link") {
            match parse_link_header::parse(link_header) {
                Ok(rels) => {
                    next_url = rels
                        .get(&Some("next".to_string()))
                        .map(|l| l.uri.to_string());
                }
                Err(_) => {
                    warn!("Failed to parse link header!");
                    next_url = None;
                }
            }
        }

        results.push(res.into_json_deserialize()?);
    }

    Ok(results)
}

fn check_ratelimit(res: &ureq::Response) {
    if let Some(remaining) = res.header("X-RateLimit-Remaining") {
        match remaining.parse::<u32>() {
            Ok(remaining) => {
                if remaining == 0 {
                    error!("Github API RateLimit exceeded!");
                    if let Some(resets_at) = res.header("X-RateLimit-Reset") {
                        match resets_at.parse::<i64>() {
                            Ok(resets_at) => info!(
                                "RateLimit resets in {} minutes",
                                (resets_at - Utc::now().timestamp()) / 60
                            ),
                            Err(e) => warn!("Failed to parse X-RateLimit-Reset Header: {}", e),
                        };
                    }
                } else if remaining < 10 {
                    warn!("Only {} github API requests remaining", remaining)
                }
            }
            Err(e) => warn!("Failed to parse X-RateLimit-Remaining Header: {}", e),
        }
    }
}

pub fn download(url: &str, name: &str, folder: &PathBuf) -> Result<PathBuf> {
    let mut output_path = folder.clone();
    output_path.push(name);

    info!("Downloading {} to {}", url, name);
    let mut output_file = File::create(&output_path)?;
    let res = ureq::get(url).call();
    let bufreader = BufReader::with_capacity(128 * 1024, res.into_reader());

    let total = Arc::new(AtomicUsize::new(0));
    let done = Arc::new(AtomicBool::new(false));
    let mut reader = ProgressReader::new(bufreader, |progress| {
        total.fetch_add(progress, Ordering::SeqCst);
    });

    let thread_total = total.clone();
    let thread_done = done.clone();
    thread::spawn(move || loop {
        if thread_done.load(Ordering::SeqCst) {
            break;
        }
        info!(
            "Downloaded {} KiB",
            thread_total.load(Ordering::SeqCst) / 1024
        );
        thread::sleep(Duration::from_secs(2));
    });

    copy(&mut reader, &mut output_file)?;
    done.store(true, Ordering::SeqCst);

    info!("Download finished");
    Ok(output_path)
}
