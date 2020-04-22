use crate::{archive, github};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

pub fn install(destination: PathBuf) -> Result<(), Box<dyn Error>> {
    info!("Installing to {}", destination.to_string_lossy());
    fs::create_dir_all(&destination)?;

    let assets = github::get_release_assets()?;

    let asset_marker: &str;
    if cfg!(windows) {
        asset_marker = "win64";
    } else if cfg!(unix) {
        asset_marker = "x86_64-continuous.tar.gz"; // Don't match the AppImage
    } else {
        asset_marker = "macos";
    }
    for asset in assets {
        if asset.name.contains(asset_marker) {
            info!("Choosing asset with name {}", asset.name);
            let archive_file = github::download(&asset, destination.clone())?;
            archive::unpack(&archive_file, &destination);
        }
    }
    info!("Done!");
    Ok(())
}
