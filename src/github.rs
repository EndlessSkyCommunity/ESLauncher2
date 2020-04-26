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
    pub name: String,
    pub updated_at: DateTime<Utc>,
    pub browser_download_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PR {
    head: PRHead,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PRHead {
    label: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnblockedArtifact {
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkflowRunArtifact {
    id: u16,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct WorkflowRunArtifacts {
    artifacts: Vec<WorkflowRunArtifact>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Workflows {
    workflows: Vec<Workflow>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Workflow {
    name: String,
    id: u16,
}

pub fn get_pr(id: u16) -> Result<PR, Error> {
    let res = ureq::get(&format!(
        "https://api.github.com/repos/endless-sky/endless-sky/pulls/{}",
        id
    ))
    .set("User-Agent", "ESLauncher2")
    .call();
    let pr: PR = serde_json::from_value(res.into_json()?)?;
    Ok(pr)
}

pub fn unblock_artifact_download(artifact_id: u16) -> Result<UnblockedArtifact, Error> {
    let res = ureq::get(&format!(
        "https://endlesssky.mcofficer.me/actions-artifacts/artifact/{}",
        artifact_id
    ))
    .set("User-Agent", "ESLauncher2")
    .call();
    let artifact: UnblockedArtifact = serde_json::from_value(res.into_json()?)?;
    Ok(artifact)
}
pub fn get_cd_workflow() -> Result<Workflow, Error> {
    let res = ureq::get("https://api.github.com/repos/endless-sky/endless-sky/actions/workflows")
        .set("User-Agent", "ESLauncher2")
        .call();
    let workflows: Workflows = serde_json::from_value(res.into_json()?)?;
    for workflow in workflows.workflows {
        if workflow.name.eq("CD") {
            return Ok(workflow);
        }
    }
    Err(Error::new(
        ErrorKind::NotFound,
        "Failed to find artifact with name \"CD\"",
    ))
}

pub fn get_workflow_run_artifacts(run_id: u16) -> Result<Vec<WorkflowRunArtifact>, Error> {
    let res = ureq::get(&format!(
        "https://api.github.com/repos/endless-sky/endless-sky/actions/runs/{}/artifacts",
        run_id
    ))
    .set("User-Agent", "ESLauncher2")
    .call();
    let artifacts: WorkflowRunArtifacts = serde_json::from_value(res.into_json()?)?;
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

pub fn download(asset: &ReleaseAsset, folder: PathBuf) -> Result<PathBuf, Error> {
    let mut output_path = folder;
    output_path.push(&asset.name);

    info!(
        "Downloading {} to {}",
        asset.browser_download_url, asset.name
    );
    let mut output_file = File::create(&output_path)?;
    let res = ureq::get(&asset.browser_download_url).call();
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
