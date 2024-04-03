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
            error!("Music thread crashed: {:#}", e);
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
                    fade(&sink, true);
                }
                MusicCommand::Play => {
                    state = MusicState::Playing;
                    fade(&sink, false);
                }
                MusicCommand::WeakPause => fade(&sink, true),
                MusicCommand::WeakPlay => {
                    if state == MusicState::Playing {
                        fade(&sink, false);
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

fn fade(sink: &Sink, out: bool) {
    let mut range: Vec<i32> = (1..20).collect();
    if out {
        range.reverse();
    } else {
        sink.play();
    }

    for i in range {
        sink.set_volume(i as f32 / 20.);
        thread::sleep(Duration::from_millis(20));
    }

    if out {
        sink.pause();
    }
}
