use crate::github::{get_workflow_run_artifacts, Artifact};
use crate::install_frame::{InstanceSource, InstanceSourceType};
use crate::instance::{Instance, InstanceType};
use crate::{archive, github};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, io, io::Write};

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
        // MacOS get's a disk image, which the regular unpacker doesn't know and causes him to panic
        if !cfg!(target_os = "macos") {
            archive::unpack(&archive_file, &destination)?;
        }    
    }

    if cfg!(target_os = "linux") {
        chmod_x(&executable_path);
    }

    if cfg!(target_os = "macos") {
        mac_postprocess(archive_file.to_string_lossy().to_string());
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
    archive::unpack(&archive_path, destination)?;
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

fn chmod_x(file: &PathBuf) {
    info!("Running chmod +x {}", file.to_string_lossy());
    if let Err(e) = Command::new("/usr/bin/chmod").arg("+x").arg(file).output() {
        error!("Failed to run chmod +x: {}", e)
    };
}

fn mac_postprocess(archive_path: String) {
    info!("Mac postprocessing starting...");
    
    // Mount the disk image file
    let archive_path_wq = format!("{}", archive_path);
    info!("  Mounting dmg file {}", archive_path_wq.clone());
    let output = Command::new("/usr/bin/hdiutil")
                            .arg("attach")
                            .arg(archive_path_wq.clone())
                            .output()
                            .expect("Mount failed");
    info!("  Result of mount: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    // Now copy the app via script, because it's not possible to do this from the rust runtime
    // we need the stem of the archive name, because this is the name under which MacOS mounts
    // and the target is the instance location
    let mount_name = Path::new(&archive_path).file_stem().unwrap().to_str().unwrap();
    let archive_parent = Path::new(&archive_path).parent().unwrap().to_str().unwrap();
    let app_source_path = format!("/Volumes/{}/Endless Sky.app", mount_name);
    let app_target_path = format!("{}/", archive_parent);
    info!("  Calling copy script with parameters:");
    info!("    {}", app_source_path.clone());
    info!("    {}", app_target_path.clone());
    let output = Command::new("./ESLauncher2_copy.sh")
                            .arg(app_source_path.clone())
                            .arg(app_target_path.clone())
                            .output()
                            .expect("Copy failed");
    info!("  Result of copy: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    // detach the drive
    let detach_path = format!("/Volumes/{}", mount_name);
    info!("  Detaching {}", detach_path.clone());
    let output = Command::new("/usr/bin/hdiutil")
                            .arg("detach")
                            .arg(detach_path.clone())
                            .output()
                            .expect("Detach failed");
    info!("  Result of detach: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
    info!("Mac postprocessing done...");

    // for now missing: delete the dmg file
}
