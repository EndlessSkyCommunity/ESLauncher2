use anyhow::Result;
use flate2::read::GzDecoder;
use std::ffi::OsStr;
use std::fs::{create_dir, File};
use std::path::PathBuf;
use tar::Archive;

/// Unpacks a .tar.gz or .zip archive.
/// If `strip_toplevel` is true, zip archives containing a single folder will extract the contents of that folder instead.
/// .tar.gz archives are not affected by `strip_toplevel`.
pub fn unpack(archive_file: &PathBuf, destination: &PathBuf, strip_toplevel: bool) -> Result<()> {
    info!(
        "Extracting {} to {}",
        archive_file.to_string_lossy(),
        destination.to_string_lossy()
    );
    match archive_file.extension().and_then(OsStr::to_str) {
        Some("gz") => {
            let file = File::open(archive_file)?;
            let decompressed = GzDecoder::new(file);
            let mut a = Archive::new(decompressed);
            a.unpack(destination)?;
        }
        Some("zip") => {
            if !destination.exists() {
                create_dir(destination)?;
            }
            espim::unzip(&destination, std::fs::read(archive_file)?, strip_toplevel)?;
        }
        _ => panic!("Unsupported archive!"),
    };
    Ok(())
}
