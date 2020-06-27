use anyhow::Result;
use rodio::decoder::DecoderError;
use rodio::{Device, Sink};
use std::io::{BufReader, Cursor};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

const SONG: &[u8] = include_bytes!("../assets/endless-prototype.ogg");

#[derive(Clone, Copy, Debug)]
pub enum MusicCommand {
    Pause,
    Play,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MusicState {
    Playing,
    Paused,
}

pub fn spawn() -> Sender<MusicCommand> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        if let Err(e) = play(rx) {
            error!("Music thread crashed: {}", e)
        }
    });
    tx
}

fn play(rx: Receiver<MusicCommand>) -> Result<()> {
    let device =
        rodio::default_output_device().ok_or_else(|| anyhow!("Failed to find default output"))?;

    let mut sink = play_once(&device)?;
    let mut state = MusicState::Playing;
    loop {
        if let Ok(cmd) = rx.try_recv() {
            match cmd {
                MusicCommand::Pause => {
                    state = MusicState::Paused;
                    sink.pause()
                }
                MusicCommand::Play => {
                    state = MusicState::Playing;
                    sink.play()
                }
            }
        }

        if state == MusicState::Playing && sink.empty() {
            sink = play_once(&device)?;
        }

        thread::sleep(Duration::from_millis(100));
    }
}

fn play_once(device: &Device) -> Result<Sink, DecoderError> {
    let reader = BufReader::new(Cursor::new(SONG));
    rodio::play_once(&device, reader)
}
