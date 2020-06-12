use anyhow::Result;
use std::io::{Cursor, Read};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::{fs, io};

pub(crate) fn download(url: &str) -> Result<Vec<u8>> {
    let resp = ureq::get(url).call();
    if resp.error() {
        return Err(anyhow!("Got bad status code {}", resp.status()));
    }

    let mut reader = io::BufReader::new(resp.into_reader());
    let mut bytes = vec![];
    reader.read_to_end(&mut bytes)?;

    Ok(bytes)
}

/// Unzips a zip archive to `destination`. If the archive contains only a top-level
/// directory, the everything inside it will be extracted. If `strip_toplevel` is true,
/// archives containing a single folder will extract the contents of that folder instead.
pub fn unzip(destination: &PathBuf, bytes: Vec<u8>, strip_toplevel: bool) -> Result<()> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))?;
    let should_strip_toplevel = strip_toplevel && has_toplevel(&mut archive);

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let mut outpath = destination.clone();
        let mut archive_path = file.sanitized_name();
        if should_strip_toplevel {
            archive_path = strip_toplevel_dir(archive_path)?;
        }
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

        if file.name().ends_with('/') {
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

        #[cfg(unix)]
        set_unix_mode(&file, &outpath)
    }
    Ok(())
}

#[cfg(unix)]
fn set_unix_mode(file: &zip::read::ZipFile, outpath: &PathBuf) {
    if let Some(m) = file.unix_mode() {
        if let Err(e) = fs::set_permissions(&outpath, PermissionsExt::from_mode(m)) {
            warn!(
                "Failed to set permissions for {}: {}",
                outpath.to_string_lossy(),
                e
            )
        }
    }
}

fn has_toplevel(archive: &mut zip::ZipArchive<Cursor<Vec<u8>>>) -> bool {
    let mut toplevel_dir: Option<PathBuf> = None;
    if archive.len() < 2 {
        return false;
    }

    for i in 0..archive.len() {
        let file = archive.by_index(i).unwrap().sanitized_name();
        if let Some(toplevel_dir) = &toplevel_dir {
            if !file.starts_with(toplevel_dir) {
                return false;
            }
        } else {
            // First iteration
            toplevel_dir = Some(file.components().take(1).collect());
        }
    }
    true
}

fn strip_toplevel_dir(archive_path: PathBuf) -> Result<PathBuf> {
    let base = archive_path
        .components()
        .take(1)
        .fold(PathBuf::new(), |mut p, c| {
            p.push(c);
            p
        });
    Ok(archive_path.strip_prefix(base)?.to_path_buf())
}
