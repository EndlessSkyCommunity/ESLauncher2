use crate::instance::{Instance, InstanceType};
use crate::{archive, github};
use std::error::Error;
use std::path::PathBuf;
use std::process::Command;
use std::{fs, io};

pub fn install(
    destination: PathBuf,
    name: String,
    instance_type: InstanceType,
) -> Result<Instance, Box<dyn Error>> {
    info!("Installing to {}", destination.to_string_lossy());
    if let InstanceType::Unknown = instance_type {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Cannot install InstanceType::Unknown",
        )));
    }

    fs::create_dir_all(&destination)?;

    let assets = github::get_release_assets()?;

    for asset in assets {
        if asset.name.contains(instance_type.archive().unwrap()) {
            info!("Choosing asset with name {}", asset.name);
            let archive_file = github::download(&asset, destination.clone())?;

            if let InstanceType::AppImage = instance_type {
                // Awkward way to invert an if let...https://github.com/rust-lang/rfcs/issues/2616
            } else {
                archive::unpack(&archive_file, &destination);
            }

            let mut executable_path = destination.clone();
            executable_path.push(instance_type.executable().unwrap());

            if cfg!(unix) {
                chmod_x(&executable_path);
            }
            info!("Done!");
            return Ok(Instance::new(
                destination,
                executable_path,
                name,
                instance_type,
            ));
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
