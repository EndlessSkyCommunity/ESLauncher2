use crate::{install, style, update, Message};
use chrono::{DateTime, Local};
use iced::{button, Align, Button, Element, Row, Text};
use platform_dirs::{AppDirs, AppUI};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;
use std::{fs, thread};

pub const EXECUTABLE_NAMES: [&str; 3] = [
    "EndlessSky.exe",
    "endless-sky",
    "endless-sky-x86_64-continuous.AppImage",
];

pub const ARCHIVE_NAMES: [&str; 4] = [
    "endless-sky-x86_64-continuous.tar.gz",
    "endless-sky-x86_64-continuous.AppImage",
    "EndlessSky-win64-continuous.zip",
    "EndlessSky-macOS-continuous.zip",
];

#[derive(Debug, Clone, Copy)]
pub enum InstanceType {
    MacOS,
    Windows,
    Linux,
    AppImage,
    Unknown,
}

impl InstanceType {
    pub fn archive(&self) -> Option<&str> {
        match self {
            InstanceType::MacOS => Some("EndlessSky-macOS-continuous.zip"),
            InstanceType::Windows => Some("EndlessSky-win64-continuous.zip"),
            InstanceType::Linux => Some("endless-sky-x86_64-continuous.tar.gz"),
            InstanceType::AppImage => Some("endless-sky-x86_64-continuous.AppImage"),
            InstanceType::Unknown => None,
        }
    }

    pub fn executable(&self) -> Option<&str> {
        match self {
            InstanceType::MacOS => Some("Endless Sky.app/Content/MacOS/Endless Sky"),
            InstanceType::Windows => Some("EndlessSky.exe"),
            InstanceType::Linux => Some("endless-sky"),
            InstanceType::AppImage => Some("endless-sky-x86_64-continuous.AppImage"),
            InstanceType::Unknown => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Instance {
    play_button: button::State,
    update_button: button::State,
    delete_button: button::State,
    pub path: PathBuf,
    pub executable: PathBuf,
    pub name: String,
    pub instance_type: InstanceType,
}

impl Default for Instance {
    fn default() -> Self {
        Instance {
            play_button: button::State::default(),
            update_button: button::State::default(),
            delete_button: button::State::default(),
            path: Default::default(),
            executable: Default::default(),
            name: Default::default(),
            instance_type: InstanceType::Unknown,
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
    pub fn new(
        path: PathBuf,
        executable: PathBuf,
        name: String,
        instance_type: InstanceType,
    ) -> Self {
        Instance {
            play_button: button::State::default(),
            update_button: button::State::default(),
            delete_button: button::State::default(),
            path,
            executable,
            name,
            instance_type,
        }
    }

    pub fn update(&mut self, message: InstanceMessage) -> iced::Command<Message> {
        match message {
            InstanceMessage::Play => iced::Command::perform(
                play(
                    self.path.clone(),
                    self.executable.clone(),
                    self.name.clone(),
                ),
                Message::Dummy,
            ),
            InstanceMessage::Update => {
                iced::Command::perform(perform_update(self.path.clone()), Message::Dummy)
            }
            InstanceMessage::Delete => {
                iced::Command::perform(delete(self.path.clone()), Message::Deleted)
            }
        }
    }

    pub fn view(&mut self) -> Element<InstanceMessage> {
        Row::new()
            .spacing(10)
            .padding(10)
            .align_items(Align::Start)
            .push(Text::new(&self.name).size(24))
            .push(
                Row::new()
                    .spacing(10)
                    .push(
                        Button::new(&mut self.play_button, style::play_icon())
                            .style(style::Button::Icon)
                            .on_press(InstanceMessage::Play),
                    )
                    .push(
                        Button::new(&mut self.update_button, style::update_icon())
                            .style(style::Button::Icon)
                            .on_press(InstanceMessage::Update),
                    )
                    .push(
                        Button::new(&mut self.delete_button, style::delete_icon())
                            .style(style::Button::Destructive)
                            .on_press(InstanceMessage::Delete),
                    ),
            )
            .into()
    }
}

pub async fn perform_install(
    path: PathBuf,
    name: String,
    instance_type: InstanceType,
) -> Option<Instance> {
    match install::install(path, name, instance_type) {
        Ok(instance) => Some(instance),
        Err(e) => {
            error!("Install failed: {}", e);
            None
        }
    }
}

pub async fn delete(path: PathBuf) -> Option<PathBuf> {
    match std::fs::remove_dir_all(&path) {
        Ok(_) => {
            info!("Removed {}", path.to_string_lossy());
            Some(path)
        }
        Err(_) => {
            error!("Failed to remove {}", path.to_string_lossy());
            None
        }
    }
}

pub async fn perform_update(path: PathBuf) {
    // Yes, this is terrible. Sue me. Bitar's objects don't implement Send, and i cannot figure out
    // how to use them in the default executor (which is multithreaded, presumably). Since we don't
    // need any sort of feedback other than logs, we can just update in new, single-threaded runtime.
    thread::spawn(move || {
        match tokio::runtime::Runtime::new() {
            Ok(mut runtime) => {
                if let Err(e) = runtime.block_on(update::update_instance(path)) {
                    error!("Failed to update instance: {}", e)
                }
            }
            Err(e) => error!("Failed to spawn tokio runtime: {}", e),
        };
    });
}

pub async fn play(path: PathBuf, executable: PathBuf, name: String) {
    let mut log_path = path;
    log_path.push("logs");
    fs::create_dir_all(&log_path).unwrap();

    let time = DateTime::<Local>::from(SystemTime::now()).to_rfc3339();
    let mut out_path = log_path.clone();
    out_path.push(format!("{}.out", time));
    let mut out = File::create(out_path).unwrap();

    let mut err_path = log_path.clone();
    err_path.push(format!("{}.err", time));
    let mut err = File::create(err_path).unwrap();

    info!("Launching {}", name);
    match Command::new(&executable).output() {
        Ok(output) => {
            info!("{} exited with {}", name, output.status);
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
                                                    let instance_type = if cfg!(windows) {
                                                        InstanceType::Windows
                                                    } else if cfg!(unix) {
                                                        if executable.ends_with("AppImage") {
                                                            InstanceType::AppImage
                                                        } else {
                                                            InstanceType::Linux
                                                        }
                                                    } else {
                                                        InstanceType::MacOS
                                                    };
                                                    vec.push(Instance::new(
                                                        entry.path(),
                                                        executable,
                                                        name.to_string(),
                                                        instance_type,
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
