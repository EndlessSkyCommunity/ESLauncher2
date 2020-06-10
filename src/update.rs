use crate::github::Artifact;
use crate::install_frame::InstanceSourceType;
use crate::instance::{Instance, InstanceType};
use crate::{archive, github, install, jenkins};
use anyhow::Result;
use bitar::{clone_from_archive, clone_in_place, Archive, CloneOptions, ReaderRemote};
use std::path::PathBuf;
use tokio::fs::OpenOptions;

pub async fn update_instance(instance: Instance) -> Result<Instance> {
    if let InstanceType::Unknown = instance.instance_type {
        return Err(anyhow!("Cannot update InstanceType::Unknown",));
    }

    let mut archive_path = if InstanceType::AppImage == instance.instance_type {
        instance.executable.clone()
    } else {
        find_archive_path(&instance.path, instance.instance_type)?
    };
    if !archive_path.exists() {
        return Err(anyhow!("{} doesn't exist", archive_path.to_string_lossy()));
    }

    let new_instance = if InstanceSourceType::Continuous == instance.source.r#type {
        update_continuous_instance(&instance, &mut archive_path).await?
    } else {
        let version = if InstanceSourceType::PR == instance.source.r#type {
            github::get_pr(instance.source.identifier.parse()?)?
                .head
                .sha
        } else {
            // InstanceSourceType::Release
            github::get_latest_release("endless-sky/endless-sky")?.tag_name
        };
        if version.eq(&instance.version) {
            return Err(anyhow!("Latest version is already installed"));
        }
        info!(
            "Incremental update isn't supported for this InstanceSourceType, triggering reinstall"
        );
        install::install(
            instance.path.clone(),
            instance.name,
            instance.instance_type,
            instance.source,
        )?
    };

    info!("Done!");
    Ok(new_instance)
}

fn find_archive_path(instance_path: &PathBuf, instance_type: InstanceType) -> Result<PathBuf> {
    let mut p = instance_path.clone();
    let matcher = instance_type
        .archive()
        .ok_or_else(|| anyhow!("Got InstanceType without archive property"))?;

    for r in instance_path.read_dir()? {
        let candidate = r?.path();
        if candidate.to_string_lossy().contains(matcher) {
            p.push(candidate);
            return Ok(p);
        }
    }
    Err(anyhow!("Failed to find local instance"))
}

async fn update_continuous_instance(
    instance: &Instance,
    archive_path: &mut PathBuf,
) -> Result<Instance> {
    let version = jenkins::get_latest_sha()?;
    if version.eq(&instance.version) {
        return Err(anyhow!("Latest version is already installed"));
    }

    let artifacts = jenkins::get_latest_artifacts()?;
    let artifact = install::choose_artifact(artifacts, instance.instance_type)?;

    let url = format!(
        "https://ci.mcofficer.me/job/EndlessSky-continuous-bitar/lastBuild/artifact/{}",
        artifact.name()
    );

    // We need a tokio Runtime because, apparently, OpenOptions and friends are doing some tokio-specific stuff
    // under the hood. This isn't actually blocking in that it doesn't block the application thread.
    match tokio::runtime::Runtime::new() {
        Ok(mut runtime) => {
            if let Err(e) = runtime.block_on(bitar_update_archive(&archive_path, url)) {
                error!("Failed to update instance: {:#}", e)
            }
        }
        Err(e) => error!("Failed to spawn tokio runtime: {}", e),
    };

    if !archive_path
        .to_string_lossy()
        .ends_with(InstanceType::AppImage.archive().unwrap())
    {
        archive::unpack(&archive_path, &instance.path, true)?;
    }

    let mut new_instance = instance.clone();
    new_instance.version = version;
    Ok(new_instance)
}

async fn bitar_update_archive(target_path: &PathBuf, url: String) -> Result<()> {
    info!("Updating {} from {}", target_path.to_string_lossy(), url);
    let mut target = OpenOptions::new()
        .read(true)
        .create(true)
        .write(true)
        .open(&target_path)
        .await?;

    let mut reader = ReaderRemote::from_url(url.parse()?);
    let archive = Archive::try_init(&mut reader).await?;
    let mut chunks_left = archive.source_index().clone();

    // Build an index of the output file's chunks
    info!(
        "Updating chunks of {} in-place",
        target_path.to_string_lossy()
    );
    let used_from_self = clone_in_place(
        &CloneOptions::default(),
        &mut chunks_left,
        &archive,
        &mut target,
    )
    .await?;
    info!("Used {}b from existing file", used_from_self);

    // Read the rest from archive
    info!("Fetching {} chunks from {}", chunks_left.len(), url);
    let total_read_from_remote = clone_from_archive(
        &CloneOptions::default(),
        &mut reader,
        &archive,
        &mut chunks_left,
        &mut target,
    )
    .await?;
    info!("Used {}b from remote", total_read_from_remote,);
    Ok(())
}
