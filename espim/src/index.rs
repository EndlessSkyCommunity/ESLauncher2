use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::Read;
use std::path::PathBuf;
use std::{fs, io};
use ureq;
use zip;

#[derive(Serialize, Deserialize)]
struct PluginIndex(Vec<AvailablePlugin>);

#[derive(Serialize, Deserialize)]
pub struct AvailablePlugin {
    name: String,
    url: String,
    version: String,
    #[serde(alias = "iconUrl")]
    icon_url: String,
    author: String,
    description: String,
}

impl AvailablePlugin {
    pub fn download_url(&self) -> String {
        format!("{}/archive/{}.zip", self.url, self.version)
    }

    pub fn download(&self, extract_to: PathBuf) -> Result<()> {
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
            let mut outpath = extract_to.clone();
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
                    println!("File {} comment: {}", i, comment);
                }
            }

            if (&*file.name()).ends_with('/') {
                println!(
                    "File {} extracted to \"{}\"",
                    i,
                    outpath.as_path().display()
                );
                fs::create_dir_all(&outpath).unwrap();
            } else {
                println!(
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

        Ok(())
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
    let index: PluginIndex = serde_json::from_reader(resp.into_reader())?;
    Ok(index.0)
}
