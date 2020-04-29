use crate::archive;
use crate::install_frame::InstanceSource;
use crate::instance::InstanceType;
use anyhow::Result;
use bitar::{clone_from_archive, clone_in_place, Archive, CloneOptions, ReaderRemote};
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use url::Url;

pub async fn update_instance(
    path: PathBuf,
    instance_type: InstanceType,
    source: InstanceSource,
) -> Result<()> {
    if let InstanceType::Unknown = instance_type {
        return Err(anyhow!("Cannot install InstanceType::Unknown",));
    }
    if let InstanceSource::PR { .. } = source {
        return Err(anyhow!("Updates are not yet supported for PRs!"));
    }

    let mut archive_path = path.clone();
    archive_path.push(instance_type.archive().unwrap());
    if archive_path.exists() {
        let url = format!(
            "https://ci.mcofficer.me/job/EndlessSky-continuous-bitar/lastBuild/artifact/{}.cba",
            instance_type.archive().unwrap()
        );
        bitar_update_archive(&archive_path, url).await?;

        if !archive_path.ends_with(InstanceType::AppImage.archive().unwrap()) {
            archive::unpack(&archive_path, &path)?;
        }
        info!("Done!");
        return Ok(());
    }
    error!("{} doesn't exist", archive_path.to_string_lossy());
    Ok(())
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
