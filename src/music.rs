use std::io::{BufReader, Cursor};
use std::panic;

pub fn play() -> Option<()> {
    panic::catch_unwind(|| {
        let device = rodio::default_output_device().unwrap();
        let song: &[u8] = include_bytes!("../assets/endless-prototype.ogg");
        let reader = BufReader::new(Cursor::new(song));
        rodio::play_once(&device, reader).unwrap().detach();
    })
    .ok()
}
