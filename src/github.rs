use crate::instance::Progress;
use crate::send_progress_message;
use anyhow::Result;
use progress_streams::ProgressReader;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::fs::File;
use std::io::{copy, BufReader};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use time::OffsetDateTime;

#[derive(Deserialize, Debug)]
pub struct Repo {
    pub(crate) id: u32,
}

pub trait Artifact {
    fn name(&self) -> &str;

    fn expired(&self) -> bool;
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

pub fn unblock_artifact_download(artifact_id: u32) -> String {
    format!(
        "https://artifact-unblocker.mcofficer.workers.dev/artifact/{}",
        artifact_id,
    )
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
    pub(crate) id: u64,
    run_number: u32,
    head_repository: Option<Repo>,
}

pub fn get_latest_workflow_run(
    workflow_id: u32,
    branch: &str,
    head_repo_id: u32,
) -> Result<WorkflowRun> {
    let mut pages: Vec<WorkflowRuns> = make_paginated_request(&format!(
        "https://api.github.com/repos/endless-sky/endless-sky/actions/workflows/{}/runs?branch={}",
        workflow_id, branch
    ))?;

    let runs: Vec<WorkflowRun> = pages
        .drain(..)
        .flat_map(|runs| runs.workflow_runs)
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
    pub size_in_bytes: u32,
    name: String,
    expired: bool,
}

impl Artifact for WorkflowRunArtifact {
    fn name(&self) -> &str {
        &self.name
    }

    fn expired(&self) -> bool {
        self.expired
    }
}

pub fn get_workflow_run_artifacts(run_id: u64) -> Result<Vec<WorkflowRunArtifact>> {
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
}

pub fn get_release_by_tag(tag: &str) -> Result<Release> {
    make_request(&format!(
        "https://api.github.com/repos/endless-sky/endless-sky/releases/tags/{}",
        tag
    ))
}

pub fn get_latest_release(repo_slug: &str) -> Result<String> {
    let url = &format!("https://github.com/{}/releases/latest", repo_slug);
    let res = ureq::get(url).call()?;

    if res.status() >= 400 {
        warn!(
            "Got unexpected status code '{} {}' for {}",
            res.status(),
            res.status_text(),
            url,
        )
    };

    Ok(res.get_url().rsplit_once('/').unwrap().1.to_string())
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

    fn expired(&self) -> bool {
        false
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
    debug!("Requesting {}", url);
    let res = ureq::get(url).set("User-Agent", "ESLauncher2").call()?;
    check_ratelimit(&res);
    if res.status() >= 400 {
        warn!(
            "Got unexpected status code '{} {}' for {}",
            res.status(),
            res.status_text(),
            url,
        )
    }
    Ok(res.into_json()?)
}

fn make_paginated_request<T: DeserializeOwned>(url: &str) -> Result<Vec<T>> {
    let mut next_url = Some(url.to_string());
    let mut results = vec![];

    while next_url.is_some() {
        let url = next_url.clone().unwrap();
        debug!("Requesting {}", url);
        let res = ureq::get(&url).set("User-Agent", "ESLauncher2").call()?;
        check_ratelimit(&res);

        if let Some(link_header) = res.header("link") {
            if let Ok(rels) = parse_link_header::parse(link_header) {
                next_url = rels
                    .get(&Some("next".to_string()))
                    .map(|l| l.uri.to_string());
            } else {
                warn!("Failed to parse link header!");
                next_url = None;
            }
        } else {
            next_url = None;
        }

        results.push(res.into_json()?);
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
                                (resets_at - OffsetDateTime::now_utc().unix_timestamp()) / 60
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

pub fn download(
    instance_name: &str,
    url: &str,
    name: &str,
    folder: &Path,
    size_hint: Option<u32>,
) -> Result<PathBuf> {
    let mut output_path = folder.to_path_buf();
    output_path.push(name);
    let mut output_file = File::create(&output_path)?;

    info!("Downloading {} to {}", url, name);
    send_progress_message(instance_name, "Downloading".into());

    let res = ureq::get(url).call()?;
    let total: Option<u32> = res
        .header("Content-Length")
        .and_then(|s| s.parse().ok())
        .or(size_hint);
    let fetched = Arc::new(AtomicUsize::new(0));
    let finished = Arc::new(AtomicBool::new(false));

    let bufreader = BufReader::with_capacity(128 * 1024, res.into_reader());
    let mut reader = ProgressReader::new(bufreader, |progress| {
        fetched.fetch_add(progress, Ordering::SeqCst);
    });

    let thread_fetched = fetched.clone();
    let thread_finished = finished.clone();
    let thread_instance_name = instance_name.to_string();
    thread::spawn(move || loop {
        if thread_finished.load(Ordering::SeqCst) {
            break;
        }
        let fetched = thread_fetched.load(Ordering::SeqCst);
        send_progress_message(
            &thread_instance_name,
            Progress::from("Downloading")
                .done(fetched as u32)
                .total(total)
                .units("b"),
        );
        thread::sleep(Duration::from_millis(30));
    });

    let res = copy(&mut reader, &mut output_file);
    // Make sure we end the logging thread before potentially erroring out
    finished.store(true, Ordering::SeqCst);
    // Tiny sleep to make sure we avoid a potential race condition
    thread::sleep(Duration::from_millis(20));
    res?;

    info!("Download finished");
    Ok(output_path)
}
