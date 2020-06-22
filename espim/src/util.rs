use anyhow::Result;
use std::io;
use std::io::{Cursor, Read};
use std::path::PathBuf;

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

pub fn unzip(destination: &PathBuf, bytes: Vec<u8>) -> Result<()> {
    Ok(zip_extract::extract(Cursor::new(bytes), destination, true)?)
}
