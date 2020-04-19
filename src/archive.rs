use flate2::read::GzDecoder;
use std::ffi::OsStr;
use std::fs::{create_dir, File};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use tar::Archive;
use zip_extensions::zip_extract;

pub fn unpack(sender: &Sender<String>, archive_file: &PathBuf, destination: &PathBuf) {
    sender
        .send(format!(
            "Extracting {} to {}",
            archive_file.to_string_lossy(),
            destination.to_string_lossy()
        ))
        .ok();
    match archive_file.extension().and_then(OsStr::to_str) {
        Some("gz") => {
            let file = File::open(archive_file).unwrap();
            let decompressed = GzDecoder::new(file);
            let mut a = Archive::new(decompressed);
            a.unpack(destination).unwrap();
        }
        Some("zip") => {
            create_dir(destination).unwrap();
            zip_extract(archive_file, destination).unwrap();
        }
        _ => panic!("Unsupported archive!"),
    };
}
