use crate::{es_plugin_dir, util, AvailablePlugin, InstalledPlugin};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use std::{fs, io};
use ureq;
use zip;

#[derive(Serialize, Deserialize)]
struct PluginIndex(Vec<AvailablePlugin>);

impl AvailablePlugin {
    pub fn download_url(&self) -> String {
        format!("{}/archive/{}.zip", self.url, self.version)
    }

    pub fn download(&self) -> Result<InstalledPlugin> {
        let mut destination =
            es_plugin_dir().ok_or_else(|| anyhow!("Failed to get ES Plug-In Dir"))?;
        destination.push(&self.name);

        info!(
            "Downloading {} to {}",
            self.name,
            destination.to_string_lossy()
        );

        if destination.exists() {
            fs::remove_dir_all(&destination)?;
        }
        fs::create_dir_all(&destination)?;

        let bytes = util::download(&self.download_url())?;
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let mut outpath = destination.clone();
            let archive_path = file.sanitized_name();
            let base = archive_path
                .components()
                .take(1)
                .fold(PathBuf::new(), |mut p, c| {
                    p.push(c);
                    p
                });
            let archive_path = archive_path.strip_prefix(base)?;
            if archive_path.to_string_lossy().is_empty() {
                // Top-level directory
                continue;
            }

            outpath.push(archive_path);

            {
                let comment = file.comment();
                if !comment.is_empty() {
                    debug!("File {} comment: {}", i, comment);
                }
            }

            if (&*file.name()).ends_with('/') {
                debug!(
                    "File {} extracted to \"{}\"",
                    i,
                    outpath.as_path().display()
                );
                fs::create_dir_all(&outpath).unwrap();
            } else {
                debug!(
                    "File {} extracted to \"{}\" ({} bytes)",
                    i,
                    outpath.as_path().display(),
                    file.size()
                );
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(&p).unwrap();
                    }
                }
                let mut outfile = fs::File::create(&outpath).unwrap();
                io::copy(&mut file, &mut outfile).unwrap();
            }
        }

        let mut version_file_path = destination;
        version_file_path.push(".version");
        let mut version_file = fs::File::create(version_file_path)?;
        version_file.write_all(self.version.as_bytes())?;

        info!("Done!");
        Ok(InstalledPlugin {
            name: self.name.clone(),
            version: self.version.clone(),
        })
    }
}

pub(crate) fn get_available_plugins() -> Result<Vec<AvailablePlugin>> {
    let resp = ureq::get(
        "https://github.com/EndlessSkyCommunity/endless-sky-plugins/raw/master/generated/plugins.json",
    )
    .call();
    if resp.error() {
        return Err(anyhow!("Got bad status code {}", resp.status()));
    }
    let index: PluginIndex = resp.into_json_deserialize()?;
    Ok(index.0)
}
