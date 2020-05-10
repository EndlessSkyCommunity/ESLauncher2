use anyhow::Result;
use chrono::Utc;
use progress_streams::ProgressReader;
use serde::Deserialize;
use serde_json;
use std::fs::File;
use std::io::{copy, BufReader};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Deserialize, Debug)]
pub struct GitRef {
    pub object: GitObject,
}

#[derive(Deserialize, Debug)]
pub struct GitObject {
    pub sha: String,
}

#[derive(Deserialize, Debug)]
pub struct Release {
    pub id: i64,
    pub tag_name: String,
    assets_url: String,
}

#[derive(Deserialize, Debug)]
struct ReleaseAssets(Vec<ReleaseAsset>);

#[derive(Deserialize, Debug)]
pub struct ReleaseAsset {
    pub id: i64,
    name: String,
    pub browser_download_url: String,
}

#[derive(Deserialize, Debug)]
pub struct WorkflowRunArtifact {
    pub id: u32,
    name: String,
}

#[derive(Deserialize, Debug)]
struct WorkflowRunArtifacts {
    artifacts: Vec<WorkflowRunArtifact>,
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
#[derive(Deserialize, Debug)]
pub struct Repo {
    pub(crate) id: u32,
}

#[derive(Deserialize, Debug)]
pub struct UnblockedArtifact {
    pub url: String,
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

#[derive(Deserialize, Debug)]
pub struct WorkflowRuns {
    workflow_runs: Vec<WorkflowRun>,
}

#[derive(Deserialize, Debug)]
pub struct WorkflowRun {
    pub(crate) id: u32,
    run_number: u32,
    head_repository: Repo,
}

pub trait Artifact {
    fn name(&self) -> &str;
}

impl Artifact for ReleaseAsset {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Artifact for WorkflowRunArtifact {
    fn name(&self) -> &str {
        &self.name
    }
}

pub fn get_pr(id: u16) -> Result<PR> {
    let pr: PR = serde_json::from_value(make_json_request(&format!(
        "https://api.github.com/repos/endless-sky/endless-sky/pulls/{}",
        id
    ))?)?;
    Ok(pr)
}

pub fn unblock_artifact_download(artifact_id: u32) -> Result<UnblockedArtifact> {
    let value = make_json_request(&format!(
        "https://endlesssky.mcofficer.me/actions-artifacts/artifact/{}",
        artifact_id
    ))?;
    let artifact: UnblockedArtifact = serde_json::from_value(value)?;
    info!("Got unblocked artifact URL");
    Ok(artifact)
}
pub fn get_cd_workflow() -> Result<Workflow> {
    let value = make_json_request(
        "https://api.github.com/repos/endless-sky/endless-sky/actions/workflows",
    )?;
    let workflows: Workflows = serde_json::from_value(value)?;
    for workflow in workflows.workflows {
        if workflow.name.eq("CD") {
            info!("Found workflow with name 'CD', id {}", workflow.id);
            return Ok(workflow);
        }
    }
    Err(anyhow!("Failed to find artifact with name 'CD'",))
}

pub fn get_latest_workflow_run(
    workflow_id: u32,
    branch: String,
    head_repo_id: u32,
) -> Result<WorkflowRun> {
    let value = make_json_request(&format!(
        "https://api.github.com/repos/endless-sky/endless-sky/actions/workflows/{}/runs?branch={}",
        workflow_id, branch
    ))?;
    let runs: WorkflowRuns = serde_json::from_value(value)?;
    info!(
        "Got {} runs for workflow {}",
        runs.workflow_runs.len(),
        workflow_id
    );
    runs.workflow_runs
        .into_iter()
        .filter(|run| run.head_repository.id.eq(&head_repo_id))
        .max_by_key(|run| run.run_number)
        .ok_or_else(|| anyhow!("Got no runs for workflow!"))
}

pub fn get_workflow_run_artifacts(run_id: u32) -> Result<Vec<WorkflowRunArtifact>> {
    let value = make_json_request(&format!(
        "https://api.github.com/repos/endless-sky/endless-sky/actions/runs/{}/artifacts",
        run_id
    ))?;
    let artifacts: WorkflowRunArtifacts = serde_json::from_value(value)?;
    info!(
        "Got {} artifacts for workflow run {}",
        artifacts.artifacts.len(),
        run_id
    );
    Ok(artifacts.artifacts)
}

pub fn get_release_by_tag(tag: &str) -> Result<Release> {
    let value = make_json_request(&format!(
        "https://api.github.com/repos/endless-sky/endless-sky/releases/tags/{}",
        tag
    ))?;
    let release: Release = serde_json::from_value(value)?;
    Ok(release)
}

pub fn get_git_ref(name: &str) -> Result<GitRef> {
    let value = make_json_request(&format!(
        "https://api.github.com/repos/endless-sky/endless-sky/git/ref/{}",
        name
    ))?;
    let r#ref: GitRef = serde_json::from_value(value)?;
    Ok(r#ref)
}

pub fn get_latest_release(repo_slug: &str) -> Result<Release> {
    let value = make_json_request(&format!(
        "https://api.github.com/repos/{}/releases/latest",
        repo_slug
    ))?;
    let release: Release = serde_json::from_value(value)?;
    Ok(release)
}

pub fn get_release_assets(release_id: i64) -> Result<Vec<ReleaseAsset>> {
    let value = make_json_request(&format!(
        "https://api.github.com/repos/endless-sky/endless-sky/releases/{}/assets",
        release_id
    ))?;
    let assets: ReleaseAssets = serde_json::from_value(value)?;
    info!("Got {} assets for release {}", assets.0.len(), release_id);
    Ok(assets.0)
}

fn make_json_request(url: &str) -> Result<serde_json::Value> {
    let res = ureq::get(url).set("User-Agent", "ESLauncher2").call();

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

    Ok(res.into_json()?)
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
