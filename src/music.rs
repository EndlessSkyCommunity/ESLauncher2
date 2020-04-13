use std::io::{BufReader, Cursor};

pub fn play() -> Option<()> {
    let device = rodio::default_output_device()?;
    let song: &[u8] = include_bytes!("../assets/endless-prototype.ogg");
    let reader = BufReader::new(Cursor::new(song));
    rodio::play_once(&device, reader).ok()?.detach();
    Some(())
}
