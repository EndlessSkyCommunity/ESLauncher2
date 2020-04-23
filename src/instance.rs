use crate::{style, Message};
use chrono::{DateTime, Local};
use iced::{button, Align, Button, Element, Length, Row, Space, Text};
use platform_dirs::{AppDirs, AppUI};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;

const EXECUTABLE_NAMES: [&str; 3] = [
    "EndlessSky.exe",
    "endless-sky",
    "endless-sky-x86_64-continuous.AppImage",
];

#[derive(Debug, Clone)]
pub struct Instance {
    pub path: PathBuf,
    pub executable: PathBuf,
    pub name: String,
    pub state: InstanceState,
}

#[derive(Debug, Clone)]
pub struct InstanceState {
    play_button: button::State,
    update_button: button::State,
    delete_button: button::State,
}

impl Default for InstanceState {
    fn default() -> Self {
        Self {
            play_button: button::State::default(),
            update_button: button::State::default(),
            delete_button: button::State::default(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum InstanceMessage {
    Play,
    Update,
    Delete,
}

impl Instance {
    pub fn new(path: PathBuf, executable: PathBuf, name: String) -> Self {
        Instance {
            path,
            executable,
            name,
            state: InstanceState::default(),
        }
    }

    pub fn update(&mut self, message: InstanceMessage) -> iced::Command<Message> {
        match message {
            InstanceMessage::Play => self.play(),
            InstanceMessage::Update => info!("STUB: update {}", self.name),
            InstanceMessage::Delete => info!("STUB: delete {}", self.name),
        }
        iced::Command::none()
    }

    pub fn view(&mut self) -> Element<InstanceMessage> {
        Row::new()
            .spacing(10)
            .padding(10)
            .align_items(Align::Start)
            .push(Text::new(&self.name).size(24))
            .push(Space::new(Length::Shrink, Length::Shrink))
            .push(
                Row::new()
                    .spacing(10)
                    .push(
                        Button::new(&mut self.state.play_button, style::play_icon())
                            .style(style::Button::Icon)
                            .on_press(InstanceMessage::Play),
                    )
                    .push(
                        Button::new(&mut self.state.update_button, style::update_icon())
                            .style(style::Button::Icon)
                            .on_press(InstanceMessage::Update),
                    )
                    .push(
                        Button::new(&mut self.state.delete_button, style::delete_icon())
                            .style(style::Button::Destructive)
                            .on_press(InstanceMessage::Delete),
                    ),
            )
            .into()
    }

    pub fn play(&self) {
        let mut log_path = self.path.clone();
        log_path.push("logs");
        fs::create_dir_all(&log_path).unwrap();

        let time = DateTime::<Local>::from(SystemTime::now()).to_rfc3339();
        let mut out_path = log_path.clone();
        out_path.push(format!("{}.out", time));
        let mut out = File::create(out_path).unwrap();

        let mut err_path = log_path.clone();
        err_path.push(format!("{}.err", time));
        let mut err = File::create(err_path).unwrap();

        info!("Launching {}", self.executable.to_string_lossy());
        match Command::new(&self.executable).output() {
            Ok(output) => {
                info!("Process exited with {}", output.status);
                out.write_all(&output.stdout).unwrap();
                err.write_all(&output.stderr).unwrap();
                info!(
                    "Logfiles have been written to {}",
                    log_path.to_string_lossy()
                );
                if !output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    error!("Stdout was: {}", stdout);
                    error!("Stderr was: {}", stderr);
                }
            }
            Err(e) => error!("Error starting process: {}", e),
        };
    }
}

pub fn get_instances_dir() -> Option<PathBuf> {
    let mut dir = AppDirs::new(Some("ESLauncher2"), AppUI::Graphical)?.data_dir;
    dir.push("instances");
    Some(dir)
}

pub fn scan_instances() -> Option<Vec<Instance>> {
    info!("Scanning Instances folder");
    let buf = get_instances_dir()?;
    let dir = buf.as_path();
    let mut vec = vec![];

    if dir.exists() {
        match dir.read_dir() {
            Ok(readdir) => {
                for result in readdir {
                    match result {
                        Ok(entry) => match entry.file_type() {
                            Ok(file_type) => {
                                if file_type.is_dir() {
                                    match entry.file_name().into_string() {
                                        Ok(name) => {
                                            let mut found = false;
                                            for exec_name in EXECUTABLE_NAMES.iter() {
                                                let mut executable = entry.path().clone();
                                                executable.push(exec_name);
                                                if executable.exists() {
                                                    vec.push(Instance::new(
                                                        entry.path(),
                                                        executable,
                                                        name.to_string(),
                                                    ));
                                                    found = true;
                                                    break;
                                                }
                                            }
                                            if !found {
                                                error!(
                                                    "Failed to find executable at {}",
                                                    entry.path().to_string_lossy()
                                                );
                                            }
                                        }
                                        Err(_) => error!(
                                            "Failed to convert filename of {} to String",
                                            entry.path().to_string_lossy(),
                                        ),
                                    };
                                }
                            }
                            Err(e) => error!(
                                "Failed to get filetype of {}: {}",
                                entry.path().to_string_lossy(),
                                e
                            ),
                        },
                        Err(e) => error!("Failed to read entry from instances folder: {}", e),
                    }
                }
            }
            Err(e) => error!("Failed to read from instances folder: {}", e),
        };
    } else if let Err(e) = fs::create_dir_all(dir) {
        error!("Failed to create instances dir: {}", e);
    }
    info!("Found {} Instances", vec.len());
    Some(vec)
}
