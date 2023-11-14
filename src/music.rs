use anyhow::{Context, Result};
use rodio::Sink;
use serde::{Deserialize, Serialize};
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
    WeakPause,
    WeakPlay,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MusicState {
    #[default]
    Playing,
    Paused,
}

pub fn spawn(initial_state: MusicState) -> Sender<MusicCommand> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        if let Err(e) = play(&rx, initial_state) {
            error!("Music thread crashed: {:#}", e)
        }
    });
    tx
}

fn play(rx: &Receiver<MusicCommand>, initial_state: MusicState) -> Result<()> {
    let (_stream, stream_handle) =
        rodio::OutputStream::try_default().context("Failed to get output stream")?;

    let sink = Sink::try_new(&stream_handle).context("Failed to create Sink")?;

    let mut state = initial_state;
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
                MusicCommand::WeakPause => sink.pause(),
                MusicCommand::WeakPlay => {
                    if let MusicState::Playing = state {
                        sink.play()
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
