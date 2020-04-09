mod archive;
mod github;
mod music;

use nfd2::Response;
use std::path::PathBuf;

pub fn main() {
    music::play();

    let destination = match nfd2::open_pick_folder(None).unwrap() {
        Response::Okay(file_path) => file_path,
        _ => panic!("Pick one"),
    };
    install(&destination);
}

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
