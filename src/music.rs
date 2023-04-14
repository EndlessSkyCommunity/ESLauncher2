use anyhow::{Context, Result};
use rodio::Sink;
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
    AutoPause,
    AutoPlay,
}

impl MusicCommand {
    pub fn update_state<F: FnOnce()>(self, music_state: MusicState, f: F) -> MusicState {
        match (self, music_state) {
            (MusicCommand::Pause, _) => {
                f();
                MusicState::Paused
            }
            (MusicCommand::Play, _) => {
                f();
                MusicState::Playing
            }
            (MusicCommand::AutoPause, MusicState::Playing) => {
                f();
                MusicState::AutoPaused
            }
            (MusicCommand::AutoPlay, MusicState::AutoPaused) => {
                f();
                MusicState::Playing
            }
            (MusicCommand::AutoPause | MusicCommand::AutoPlay, _) => music_state,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicState {
    Playing,
    Paused,
    AutoPaused,
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
            state = match cmd {
                MusicCommand::Pause | MusicCommand::AutoPause => cmd.update_state(state, || sink.pause()),
                MusicCommand::Play | MusicCommand::AutoPlay => cmd.update_state(state, || sink.play()),
            }
        }

        if state == MusicState::Playing && sink.empty() {
            let source = rodio::Decoder::new(BufReader::new(Cursor::new(SONG)))?;
            sink.append(source);
        }

        thread::sleep(Duration::from_millis(100));
    }
}
