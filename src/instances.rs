use crate::style;
use iced::{button, Align, Button, Element, Length, Row, Space, Text};
use platform_dirs::{AppDirs, AppUI};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Instance {
    pub path: PathBuf,
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
    pub fn new(path: PathBuf, name: String) -> Self {
        Instance {
            path,
            name,
            state: InstanceState::default(),
        }
    }

    pub fn update(&mut self, message: InstanceMessage) {
        match message {
            InstanceMessage::Play => info!("STUB: play {}", self.name),
            InstanceMessage::Update => info!("STUB: update {}", self.name),
            InstanceMessage::Delete => info!("STUB: delete {}", self.name),
        }
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
}

pub fn get_instances_dir() -> Option<PathBuf> {
    let mut dir = AppDirs::new(Some("ESLauncher2"), AppUI::Graphical)?.data_dir;
    dir.push("instances");
    Some(dir)
}

pub fn get_instances() -> Option<Vec<Instance>> {
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
                                        Ok(name) => vec.push(Instance::new(entry.path(), name)),
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
    Some(vec)
}
