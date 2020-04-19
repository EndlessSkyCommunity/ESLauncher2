use crate::{archive, github};
use std::path::PathBuf;
use std::sync::mpsc::Sender;

pub fn install(sender: Sender<String>, destination: PathBuf) {
    let assets = github::get_release_assets(&sender).expect("Failed to get Release Assets");

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
            github::download(&sender, &asset).unwrap();
            archive::unpack(&sender, &PathBuf::from(&asset.name), &destination);
        }
    }
    sender.send(String::from("Done!")).ok();
}
