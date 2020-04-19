use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::copy;
use std::io::Error;
use std::path::Path;
use std::sync::mpsc::Sender;

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

pub fn get_release_assets(sender: &Sender<String>) -> Result<Vec<GithubReleaseAsset>, Error> {
    let res =
        ureq::get("https://api.github.com/repos/endless-sky/endless-sky/releases/tags/continuous")
            .set("User-Agent", "ESLauncher2")
            .call();
    let release: GithubRelease = serde_json::from_value(res.into_json()?)?;
    sender.send(format!("{:#?}", release)).ok();

    let res = ureq::get(&format!(
        "https://api.github.com/repos/endless-sky/endless-sky/releases/{}/assets",
        release.id
    ))
    .call();

    let assets: GithubReleaseAssets = serde_json::from_value(res.into_json()?)?;
    sender.send(format!("{:#?}", assets)).ok();
    Ok(assets.0)
}

pub fn download(sender: &Sender<String>, asset: &GithubReleaseAsset) -> Result<u64, Error> {
    let output_path = Path::new(&asset.name);

    sender
        .send(format!(
            "Downloading {} to {}",
            asset.browser_download_url, asset.name
        ))
        .ok();
    let mut output_file = File::create(output_path)?;
    let res = ureq::get(&asset.browser_download_url).call();
    copy(&mut res.into_reader(), &mut output_file)
}
