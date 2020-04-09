use rust_embed::RustEmbed;
use std::io::{BufReader, Cursor};

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Asset;

pub(crate) fn play() -> Option<()> {
    let device = rodio::default_output_device()?;
    let song = Asset::get("endless-prototype.ogg")?;
    let reader = BufReader::new(Cursor::new(song.to_owned()));
    rodio::play_once(&device, reader).ok()?.detach();
    Some(())
}
