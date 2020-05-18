use crate::{es_plugin_dir, AvailablePlugin, InstalledPlugin};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::Read;
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
        let resp = ureq::get(&self.download_url()).call();
        if resp.error() {
            return Err(anyhow!("Got bad status code {}", resp.status()));
        }

        let mut reader = io::BufReader::new(resp.into_reader());
        let mut bytes = vec![];
        reader.read_to_end(&mut bytes)?;

        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let mut outpath =
                es_plugin_dir().ok_or_else(|| anyhow!("Failed to get ES Plug-In Dir"))?;
            outpath.push(&self.name);
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

        Ok(InstalledPlugin {
            name: self.name.clone(),
        })
    }
}

pub fn get_available_plugins() -> Result<Vec<AvailablePlugin>> {
    let resp = ureq::get(
        "https://raw.githubusercontent.com/MCOfficer/endless-sky-plugins/master/plugins.json",
    )
    .call();
    if resp.error() {
        return Err(anyhow!("Got bad status code {}", resp.status()));
    }
    let index: PluginIndex = resp.into_json_deserialize()?;
    Ok(index.0)
}
