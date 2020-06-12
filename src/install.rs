use crate::github::{get_workflow_run_artifacts, Artifact};
use crate::install_frame::{InstanceSource, InstanceSourceType};
use crate::instance::{Instance, InstanceType};
use crate::{archive, github};
use anyhow::Result;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::{fs, io};

pub fn install(
    destination: PathBuf,
    name: String,
    instance_type: InstanceType,
    mut instance_source: InstanceSource,
) -> Result<Instance> {
    info!("Installing to {}", destination.to_string_lossy());
    if let InstanceType::Unknown = instance_type {
        return Err(anyhow!("Cannot install InstanceType::Unknown",));
    }

    if InstanceSourceType::PR == instance_source.r#type
        && instance_source.identifier.starts_with('#')
    {
        instance_source.identifier.remove(0);
    } else if InstanceSourceType::Release == instance_source.r#type
        && !instance_source.identifier.starts_with('v')
    {
        instance_source.identifier.insert(0, 'v');
    }

    fs::create_dir_all(&destination)?;

    let (archive_file, version) = match instance_source.r#type {
        InstanceSourceType::Continuous => (
            download_release_asset("continuous", &destination, instance_type)?,
            github::get_git_ref("tags/continuous")?.object.sha,
        ),
        InstanceSourceType::Release => (
            download_release_asset(&instance_source.identifier, &destination, instance_type)?,
            String::from(&instance_source.identifier),
        ),
        InstanceSourceType::PR => download_pr_asset(
            &destination,
            instance_type,
            instance_source.identifier.parse()?,
        )?,
    };

    let mut executable_path = destination.clone();
    executable_path.push(instance_type.executable().unwrap());

    if let InstanceType::AppImage = instance_type {
        fs::rename(&archive_file, &executable_path)?;
    } else {
        archive::unpack(&archive_file, &destination, true)?;
    }

    // upload-artifact doesn't preserve permissions, so we need to set the executable bit here
    // https://github.com/actions/upload-artifact/issues/38
    if cfg!(unix) {
        if let Err(e) = chmod_x(&executable_path) {
            warn!(
                "Failed to set executable bit for {}: {}",
                executable_path.to_string_lossy(),
                e
            );
        }
    }

    info!("Done!");
    Ok(Instance::new(
        destination,
        executable_path,
        name,
        version,
        instance_type,
        instance_source,
    ))
}

fn download_release_asset(
    tag: &str,
    destination: &PathBuf,
    instance_type: InstanceType,
) -> Result<PathBuf> {
    let release = github::get_release_by_tag(tag)?;
    let assets = github::get_release_assets(release.id)?;
    let asset = choose_artifact(assets, instance_type)?;
    Ok(github::download(
        &asset.browser_download_url,
        asset.name(),
        &destination.clone(),
    )?)
}

fn download_pr_asset(
    destination: &PathBuf,
    instance_type: InstanceType,
    pr_id: u16,
) -> Result<(PathBuf, String)> {
    let pr = github::get_pr(pr_id)?;
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
    archive::unpack(&archive_path, destination, true)?;
    fs::remove_file(archive_path)?;

    let mut result_path = destination.clone();
    result_path.push(artifact.name());
    Ok((result_path, pr.head.sha))
}

pub fn choose_artifact<A: Artifact>(artifacts: Vec<A>, instance_type: InstanceType) -> Result<A> {
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
    Err(anyhow!(
        "Couldn't match any asset against {}",
        instance_type.archive().unwrap()
    ))
}

#[cfg(unix)]
fn chmod_x(file: &PathBuf) -> Result<()> {
    Ok(fs::set_permissions(
        &file,
        PermissionsExt::from_mode(0o755),
    )?)
}
