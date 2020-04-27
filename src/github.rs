use chrono::{DateTime, Utc};
use progress_streams::ProgressReader;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::{copy, BufReader};
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
struct Release {
    id: i64,
    assets_url: String,
}

#[derive(Deserialize, Debug)]
struct ReleaseAssets(Vec<ReleaseAsset>);

#[derive(Deserialize, Debug)]
pub struct ReleaseAsset {
    pub id: i64,
    name: String,
    pub updated_at: DateTime<Utc>,
    pub browser_download_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkflowRunArtifact {
    pub id: u32,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct WorkflowRunArtifacts {
    artifacts: Vec<WorkflowRunArtifact>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PR {
    pub head: PRHead,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PRHead {
    #[serde(alias = "ref")]
    pub branch: String,
    pub repo: Repo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Repo {
    pub(crate) id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnblockedArtifact {
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Workflows {
    workflows: Vec<Workflow>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Workflow {
    name: String,
    pub(crate) id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkflowRuns {
    workflow_runs: Vec<WorkflowRun>,
}

#[derive(Serialize, Deserialize, Debug)]
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

pub fn get_pr(id: u16) -> Result<PR, Error> {
    let res = ureq::get(&format!(
        "https://api.github.com/repos/endless-sky/endless-sky/pulls/{}",
        id
    ))
    .set("User-Agent", "ESLauncher2")
    .call();
    let pr: PR = serde_json::from_value(res.into_json()?)?;
    info!("Got PR: {:#?}", pr);
    Ok(pr)
}

pub fn unblock_artifact_download(artifact_id: u32) -> Result<UnblockedArtifact, Error> {
    let res = ureq::get(&format!(
        "https://endlesssky.mcofficer.me/actions-artifacts/artifact/{}",
        artifact_id
    ))
    .set("User-Agent", "ESLauncher2")
    .call();
    let artifact: UnblockedArtifact = serde_json::from_value(res.into_json()?)?;
    info!("Got unblocked artifact URL");
    Ok(artifact)
}
pub fn get_cd_workflow() -> Result<Workflow, Error> {
    let res = ureq::get("https://api.github.com/repos/endless-sky/endless-sky/actions/workflows")
        .set("User-Agent", "ESLauncher2")
        .call();
    let workflows: Workflows = serde_json::from_value(res.into_json()?)?;
    for workflow in workflows.workflows {
        if workflow.name.eq("CD") {
            info!("Found workflow with name 'CD', id {}", workflow.id);
            return Ok(workflow);
        }
    }
    Err(Error::new(
        ErrorKind::NotFound,
        "Failed to find artifact with name \"CD\"",
    ))
}

pub fn get_latest_workflow_run(
    workflow_id: u32,
    branch: String,
    head_repo_id: u32,
) -> Result<WorkflowRun, Error> {
    let res = ureq::get(&format!(
        "https://api.github.com/repos/endless-sky/endless-sky/actions/workflows/{}/runs?branch={}",
        workflow_id, branch
    ))
    .set("User-Agent", "ESLauncher2")
    .call();
    info!("Response: {:#?}", res);
    let runs: WorkflowRuns = serde_json::from_value(res.into_json()?)?;
    info!(
        "Got {} runs for workflow {}",
        runs.workflow_runs.len(),
        workflow_id
    );
    runs.workflow_runs
        .into_iter()
        .filter(|run| run.head_repository.id.eq(&head_repo_id))
        .max_by_key(|run| run.run_number)
        .ok_or_else(|| Error::new(ErrorKind::Other, "Got no runs for workflow!"))
}

pub fn get_workflow_run_artifacts(run_id: u32) -> Result<Vec<WorkflowRunArtifact>, Error> {
    let res = ureq::get(&format!(
        "https://api.github.com/repos/endless-sky/endless-sky/actions/runs/{}/artifacts",
        run_id
    ))
    .set("User-Agent", "ESLauncher2")
    .call();
    let artifacts: WorkflowRunArtifacts = serde_json::from_value(res.into_json()?)?;
    info!(
        "Got {} artifacts for workflow run {}",
        artifacts.artifacts.len(),
        run_id
    );
    Ok(artifacts.artifacts)
}

pub fn get_release_assets() -> Result<Vec<ReleaseAsset>, Error> {
    let res =
        ureq::get("https://api.github.com/repos/endless-sky/endless-sky/releases/tags/continuous")
            .set("User-Agent", "ESLauncher2")
            .call();
    let release: Release = serde_json::from_value(res.into_json()?)?;
    info!("Got release: {:#?}", release);

    let res = ureq::get(&format!(
        "https://api.github.com/repos/endless-sky/endless-sky/releases/{}/assets",
        release.id
    ))
    .call();

    let assets: ReleaseAssets = serde_json::from_value(res.into_json()?)?;
    info!("Got {} assets for release {}", assets.0.len(), release.id);
    Ok(assets.0)
}

pub fn download(url: &str, name: &str, folder: &PathBuf) -> Result<PathBuf, Error> {
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
