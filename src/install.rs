use crate::instance::Instance;
use crate::{archive, github};
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn install(
    destination: PathBuf,
    name: String,
    appimage: bool,
) -> Result<Instance, Box<dyn Error>> {
    info!("Installing to {}", destination.to_string_lossy());
    fs::create_dir_all(&destination)?;

    let assets = github::get_release_assets()?;

    let asset_marker: &str;
    let executable_name: &str;
    if cfg!(windows) {
        asset_marker = "win64";
        executable_name = "EndlessSky.exe";
    } else if cfg!(unix) {
        if appimage {
            asset_marker = "x86_64-continuous.AppImage";
            executable_name = "endless-sky-x86_64-continuous.AppImage";
        } else {
            asset_marker = "x86_64-continuous.tar.gz";
            executable_name = "endless-sky";
        }
    } else {
        asset_marker = "macos";
        executable_name = "" // Wat
    }

    for asset in assets {
        if asset.name.contains(asset_marker) {
            info!("Choosing asset with name {}", asset.name);
            let archive_file = github::download(&asset, destination.clone())?;

            if !(cfg!(unix) && appimage) {
                archive::unpack(&archive_file, &destination);
            }

            let mut executable = destination.clone();
            executable.push(executable_name);

            if cfg!(unix) {
                chmod_x(&executable);
            }
            info!("Done!");
            return Ok(Instance::new(destination, executable, name));
        }
    }
    Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Failed to find usable asset",
    )))
}

fn chmod_x(file: &PathBuf) {
    info!("Running chmod +x {}", file.to_string_lossy());
    if let Err(e) = Command::new("/usr/bin/chmod").arg("+x").arg(file).output() {
        error!("Failed to run chmod +x: {}", e)
    };
}
