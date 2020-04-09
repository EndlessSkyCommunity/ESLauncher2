use chrono::{DateTime, Utc};
use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde_derive::{Deserialize, Serialize};
use std::fs::File;
use std::io::copy;
use std::path::Path;

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

pub fn setup_client() -> Client {
    Client::builder()
        .user_agent("MCOfficer's Nightly Downloader")
        .build()
        .unwrap()
}

pub fn get_release_assets(client: &Client) -> Vec<GithubReleaseAsset> {
    let res = client
        .get("https://api.github.com/repos/MCOfficer/endless-sky/releases/tags/continuous")
        .header(
            "Authorization",
            "token 90c09eab100fc2e98170e1b23d61674d72744879",
        )
        .send()
        .unwrap();
    println!("{:#?}", res);

    let release: GithubRelease = res.json().unwrap();

    println!("{:#?}", release);

    let assets: GithubReleaseAssets = client
        .get(&format!(
            "https://api.github.com/repos/MCOfficer/endless-sky/releases/{}/assets",
            release.id
        ))
        .header(
            "Authorization",
            "token 90c09eab100fc2e98170e1b23d61674d72744879",
        )
        .send()
        .unwrap()
        .json()
        .unwrap();
    assets.0
}

pub fn download(client: &Client, asset: &GithubReleaseAsset) {
    let output_path = Path::new(&asset.name);
    println!(
        "Downloading {} to {}",
        asset.browser_download_url, asset.name
    );
    match client.get(&asset.browser_download_url).send() {
        Ok(mut response) => {
            match response.status() {
                StatusCode::OK => (),
                _ => {
                    eprintln!("Failed to fetch '{}'", asset.browser_download_url);
                    return;
                }
            }
            let mut output_file = match File::create(output_path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Failed to create file '{}': {}", output_path.display(), e);
                    return;
                }
            };
            match copy(&mut response, &mut output_file) {
                Ok(_) => println!("Successfully downloaded to {}", output_path.display()),
                Err(e) => eprintln!(
                    "Failed to download response body for '{}': {}",
                    asset.browser_download_url, e
                ),
            }
        }
        Err(e) => eprintln!("Failed to fetch '{}': {}", asset.browser_download_url, e),
    };
}
