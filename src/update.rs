use crate::install_frame::InstanceSourceType;
use crate::instance::{Instance, InstanceType};
use crate::{archive, install};
use anyhow::Result;
use bitar::{clone_from_archive, clone_in_place, Archive, CloneOptions, ReaderRemote};
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use url::Url;

pub async fn update_instance(instance: Instance) -> Result<Instance> {
    if let InstanceType::Unknown = instance.instance_type {
        return Err(anyhow!("Cannot update InstanceType::Unknown",));
    }

    let mut archive_path = instance.path.clone();
    archive_path.push(instance.instance_type.archive().unwrap());
    if !archive_path.exists() {
        return Err(anyhow!("{} doesn't exist", archive_path.to_string_lossy()));
    }

    let new_instance = match instance.source.r#type {
        InstanceSourceType::PR => {
            info!("Incremental update isn't supported for PRs, triggering reinstall");
            install::install(
                instance.path.clone(),
                instance.name,
                instance.instance_type,
                instance.source,
            )?
        }
        InstanceSourceType::Continuous => {
            let url = format!(
                "https://ci.mcofficer.me/job/EndlessSky-continuous-bitar/lastBuild/artifact/{}.cba",
                instance.instance_type.archive().unwrap()
            );
            bitar_update_archive(&archive_path, url).await?;
            if !archive_path.ends_with(InstanceType::AppImage.archive().unwrap()) {
                archive::unpack(&archive_path, &instance.path)?;
            }
            instance // TODO
        }
    };

    info!("Done!");
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

    let mut reader = ReaderRemote::new(Url::parse(&url)?, 3, None, None);
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
