use crate::{archive, github};
use std::path::PathBuf;

pub fn install(destination: &PathBuf) {
    let assets = github::get_release_assets().expect("Failed to get Release Assets");

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
            github::download(&asset).unwrap();
            archive::unpack(&PathBuf::from(&asset.name), &destination);
        }
    }
}
