use chrono::{DateTime, Utc};
use progress_streams::ProgressReader;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::Error;
use std::io::{copy, BufReader};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
struct GithubRelease {
    id: i64,
    assets_url: String,
}

#[derive(Deserialize, Debug)]
struct GithubReleaseAssets(Vec<GithubReleaseAsset>);

#[derive(Deserialize, Debug)]
pub struct GithubReleaseAsset {
    pub id: i64,
    pub name: String,
    pub updated_at: DateTime<Utc>,
    pub browser_download_url: String,
}

pub fn get_release_assets() -> Result<Vec<GithubReleaseAsset>, Error> {
    let res =
        ureq::get("https://api.github.com/repos/endless-sky/endless-sky/releases/tags/continuous")
            .set("User-Agent", "ESLauncher2")
            .call();
    let release: GithubRelease = serde_json::from_value(res.into_json()?)?;
    info!("Got release: {:#?}", release);

    let res = ureq::get(&format!(
        "https://api.github.com/repos/endless-sky/endless-sky/releases/{}/assets",
        release.id
    ))
    .call();

    let assets: GithubReleaseAssets = serde_json::from_value(res.into_json()?)?;
    info!("Got {} assets for release {}", assets.0.len(), release.id);
    Ok(assets.0)
}

pub fn download(asset: &GithubReleaseAsset, folder: PathBuf) -> Result<PathBuf, Error> {
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
        info!("Read {} KiB", thread_total.load(Ordering::SeqCst) / 1024);
        thread::sleep(Duration::from_secs(2));
    });

    copy(&mut reader, &mut output_file)?;
    done.store(true, Ordering::SeqCst);

    info!("Download finished");
    Ok(output_path)
}
