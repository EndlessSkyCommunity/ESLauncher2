use crate::github::{get_workflow_run_artifacts, Artifact};
use crate::install_frame::InstanceSource;
use crate::instance::{Instance, InstanceType};
use crate::{archive, github};
use std::error::Error;
use std::path::PathBuf;
use std::process::Command;
use std::{fs, io};

pub fn install(
    destination: PathBuf,
    name: String,
    pr_id: String,
    instance_type: InstanceType,
    instance_source: InstanceSource,
) -> Result<Instance, Box<dyn Error>> {
    info!("Installing to {}", destination.to_string_lossy());
    if let InstanceType::Unknown = instance_type {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Cannot install InstanceType::Unknown",
        )));
    }

    fs::create_dir_all(&destination)?;

    let archive_file = match instance_source {
        InstanceSource::Continuous => download_continuous_asset(&destination, instance_type)?,
        InstanceSource::PR => download_pr_asset(&destination, instance_type, pr_id)?,
    };

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
    Ok(Instance::new(
        destination,
        executable_path,
        name,
        instance_type,
    ))
}

fn download_continuous_asset(
    destination: &PathBuf,
    instance_type: InstanceType,
) -> Result<PathBuf, io::Error> {
    let assets = github::get_release_assets()?;
    let asset = choose_artifact(assets, instance_type)?;
    github::download(
        &asset.browser_download_url,
        asset.name(),
        &destination.clone(),
    )
}

fn download_pr_asset(
    destination: &PathBuf,
    instance_type: InstanceType,
    pr_id: String,
) -> Result<PathBuf, Box<dyn Error>> {
    let pr = github::get_pr(pr_id.parse::<u16>()?)?;
    let workflow = github::get_cd_workflow()?;
    let run = github::get_latest_workflow_run(workflow.id, pr.head.branch, pr.head.repo.id)?;
    let artifacts = get_workflow_run_artifacts(run.id)?;
    let artifact = choose_artifact(artifacts, instance_type)?;
    let unblocked = github::unblock_artifact_download(artifact.id)?;

    let archive_path = github::download(
        &unblocked.url,
        &format!("{}.zip", artifact.name()),
        destination,
    )?;
    archive::unpack(&archive_path, destination);
    fs::remove_file(archive_path)?;
    let mut result_path = destination.clone();
    result_path.push(artifact.name());
    Ok(result_path)
}

fn choose_artifact<A: Artifact>(
    artifacts: Vec<A>,
    instance_type: InstanceType,
) -> Result<A, io::Error> {
    for artifact in artifacts {
        let matches = artifact
            .name()
            .contains(instance_type.archive().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Got InstanceType without archive property",
                )
            })?);
        if matches {
            info!("Choosing asset with name {}", artifact.name());
            return Ok(artifact);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!(
            "Couldn't match any asset against {}",
            instance_type.archive().unwrap()
        ),
    ))
}

fn chmod_x(file: &PathBuf) {
    info!("Running chmod +x {}", file.to_string_lossy());
    if let Err(e) = Command::new("/usr/bin/chmod").arg("+x").arg(file).output() {
        error!("Failed to run chmod +x: {}", e)
    };
}
