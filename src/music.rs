use anyhow::{Context, Result};
use rodio::Sink;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

use crate::get_data_dir;

const SONG: &[u8] = include_bytes!("../assets/endless-prototype.ogg");

#[derive(Clone, Copy, Debug)]
pub enum MusicCommand {
    Pause,
    Play,
    WeakPause,
    WeakPlay,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicState {
    Playing,
    Paused,
}

pub fn spawn() -> Sender<MusicCommand> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        if let Err(e) = play(&rx) {
            error!("Music thread crashed: {:#}", e)
        }
    });
    tx
}

fn play(rx: &Receiver<MusicCommand>) -> Result<()> {
    let (_stream, stream_handle) =
        rodio::OutputStream::try_default().context("Failed to get output stream")?;

    let sink = Sink::try_new(&stream_handle).context("Failed to create Sink")?;

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
                MusicCommand::WeakPause => {
                    sink.pause()
                }
                MusicCommand::WeakPlay => {
                    match state {
                        MusicState::Playing => {
                            sink.play()
                        }
                        MusicState::Paused => {}
                    }
                }
            }
        }

        if state == MusicState::Playing && sink.empty() {
            let source = rodio::Decoder::new(BufReader::new(Cursor::new(SONG)))?;
            sink.append(source);
        }

        thread::sleep(Duration::from_millis(100));
    }
}

pub fn save_music_state(state: bool) -> Result<()> {
    let mut app_save_file =
        get_data_dir().ok_or_else(|| anyhow!("Failed to get app save dir"))?;
    app_save_file.push("application_save.json");

    let file = File::create(app_save_file)?;

    serde_json::to_writer_pretty(
        file,
        &state,
    )?;
    Ok(())
}

pub fn load_music_state() -> bool {
    let mut app_save_file =
        get_data_dir().ok_or_else(|| anyhow!("Failed to get app save dir")).unwrap();
    app_save_file.push("application_save.json");

    if app_save_file.exists() {
        let file = File::open(app_save_file).unwrap();

        let loaded: bool = serde_json::from_reader(file).unwrap_or(true);
        loaded
    } else {
        warn!("instances.json doesn't exist (yet?), commencing without loading Instances");
        true
    }
}
