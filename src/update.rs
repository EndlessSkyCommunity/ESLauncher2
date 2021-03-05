use crate::github::Artifact;
use crate::install_frame::InstanceSourceType;
use crate::instance::{Instance, InstanceType};
use crate::{archive, github, install, jenkins, send_progress_message};
use anyhow::Result;
use futures::{StreamExt, TryStreamExt};
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
        match update_continuous_instance(&instance, &mut archive_path).await {
            Ok(i) => i,
            Err(e) => {
                error!("Failed to perform incremental update: {}", e);
                info!("falling back to reinstall");
                install::install(
                    instance.path.clone(),
                    instance.name,
                    instance.instance_type,
                    instance.source,
                )?
            }
        }
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
        error!("Latest version is already installed");
        return Ok(instance.clone());
    }

    let artifacts = jenkins::get_latest_artifacts()?;
    let artifact = install::choose_artifact(artifacts, instance.instance_type)?;

    let url = format!(
        "https://ci.mcofficer.me/job/EndlessSky-continuous-bitar/lastBuild/artifact/{}",
        artifact.name()
    );

    bitar_update_archive(&instance.name, archive_path, url).await?;

    if !archive_path
        .to_string_lossy()
        .ends_with(InstanceType::AppImage.archive().unwrap())
    {
        send_progress_message(&instance.name, "Extracting archive", None);
        archive::unpack(archive_path, &instance.path, !cfg!(target_os = "macos"))?;
    }

    let mut new_instance = instance.clone();
    new_instance.version = version;
    Ok(new_instance)
}

async fn bitar_update_archive(
    instance_name: &str,
    target_path: &PathBuf,
    url: String,
) -> Result<()> {
    info!("Updating {} from {}", target_path.to_string_lossy(), url);
    info!(
        "Updating chunks of {} in-place",
        target_path.to_string_lossy()
    );

    // Open archive which source we want to clone
    let reader = bitar::ReaderRemote::from_url(url.parse()?);
    let mut source_archive = bitar::Archive::try_init(reader).await?;

    // Open our target file
    let mut target = OpenOptions::new()
        .read(true)
        .create(true)
        .write(true)
        .open(&target_path)
        .await?;

    send_progress_message(instance_name, "Scanning local chunks", None);
    // Scan the target file for chunks and build a chunk index
    let chunker = bitar::chunker::Chunker::new(source_archive.chunker_config(), &mut target);
    let mut chunk_stream = chunker.map_ok(|(offset, chunk)| (offset, chunk.verify()));
    let mut output_index = bitar::ChunkIndex::new_empty();
    while let Some(r) = chunk_stream.next().await {
        send_progress_message(
            instance_name,
            &format!(
                "Scanning local chunks ({}/~{})",
                output_index.len(),
                source_archive.total_chunks()
            ),
            Some((
                output_index.len() as f32,
                source_archive.total_chunks() as f32,
            )),
        );
        let (offset, verified) = r?;
        let (hash, chunk) = verified.into_parts();
        output_index.add_chunk(hash, chunk.len(), &[offset]);
    }

    // Create output to contain the clone of the archive's source
    let mut output = bitar::CloneOutput::new(target, source_archive.build_source_index());

    // Reorder chunks in the output
    send_progress_message(instance_name, "Reordering chunks", None);
    let reused_bytes = output.reorder_in_place(output_index).await?;
    info!("Used {}b from existing file", reused_bytes);

    // Fetch the rest of the chunks from the source archive
    let mut chunk_stream = source_archive.chunk_stream(&output.chunks());
    let mut read_from_remote = 0;
    while let Some(result) = chunk_stream.next().await {
        send_progress_message(
            instance_name,
            &format!("Fetching remote chunks({}b)", read_from_remote),
            None,
        );
        let compressed = result?;
        read_from_remote += compressed.len();
        let unverified = compressed.decompress()?;
        let verified = unverified.verify()?;
        output.feed(&verified).await?;
    }

    info!("Used {}b from remote", read_from_remote,);
    // Again, sleep to avoid a race condition (otherwise the "InstanceState changed" message could arrive after the update has already finished
    std::thread::sleep(std::time::Duration::from_millis(50));
    Ok(())
}
