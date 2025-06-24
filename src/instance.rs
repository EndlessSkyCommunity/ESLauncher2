use crate::install_frame::InstanceSource;
use crate::music::MusicCommand;
use crate::style::icon_button;
use crate::{get_data_dir, install, send_message, style, update, Message};
use anyhow::Result;
use iced::widget::{Button, Column, ProgressBar, Row, Space, Text};
use iced::{alignment, theme, Alignment, Element, Length};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use time::{format_description, OffsetDateTime};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum InstanceType {
    MacOS,
    Windows,
    Linux,
    AppImage,
    Unknown,
}

impl InstanceType {
    pub fn archive_matches(self, archive_name: &str) -> bool {
        match self {
            Self::MacOS => archive_name.contains("mac") || archive_name.ends_with(".dmg"),
            Self::Windows => archive_name.contains("win64"),
            Self::Linux => archive_name.ends_with(".tar.gz"),
            Self::AppImage => archive_name.contains(".AppImage"),
            Self::Unknown => false,
        }
    }

    pub fn executable(self) -> Option<&'static str> {
        match self {
            Self::MacOS => Some("Endless Sky.app/Contents/MacOS/Endless Sky"),
            Self::Windows => Some("Endless Sky.exe"),
            Self::Linux => Some("endless-sky"),
            Self::AppImage => Some("endless-sky.AppImage"),
            Self::Unknown => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    #[serde(skip)]
    pub state: InstanceState,

    pub path: PathBuf,
    pub executable: PathBuf,
    pub name: String,
    pub version: String,
    pub instance_type: InstanceType,
    pub source: InstanceSource,
}

#[derive(Debug, Clone, Default)]
pub enum InstanceState {
    Playing,
    Working(Progress),
    #[default]
    Ready,
}

#[derive(Debug, Clone, Default)]
pub struct Progress {
    status: String,
    done: Option<u32>,
    total: Option<u32>,
    units: Option<String>,
    total_approx: bool,
}

impl Progress {
    pub fn total(mut self, total: impl Into<Option<u32>>) -> Self {
        self.total = total.into();
        self
    }
    pub fn done(mut self, done: impl Into<Option<u32>>) -> Self {
        self.done = done.into();
        self
    }
    pub fn units<T: AsRef<str>>(mut self, units: T) -> Self {
        self.units = Some(units.as_ref().into());
        self
    }
    pub fn total_approx(mut self, total_approx: bool) -> Self {
        self.total_approx = total_approx;
        self
    }
}

impl<T: AsRef<str>> From<T> for Progress {
    fn from(status: T) -> Self {
        Self {
            status: status.as_ref().into(),
            ..Default::default()
        }
    }
}

impl InstanceState {
    pub fn is_playing(&self) -> bool {
        matches!(self, Self::Playing)
    }
    pub fn is_working(&self) -> bool {
        matches!(self, Self::Working { .. })
    }
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready)
    }
}

#[derive(Debug, Clone)]
pub enum InstanceMessage {
    Play(bool),
    Update,
    Folder,
    Delete,
    StateChanged(InstanceState),
}

impl Instance {
    pub fn new(
        path: PathBuf,
        executable: PathBuf,
        name: String,
        version: String,
        instance_type: InstanceType,
        source: InstanceSource,
        state: InstanceState,
    ) -> Self {
        Self {
            state,
            path,
            executable,
            name,
            version,
            instance_type,
            source,
        }
    }

    pub fn update(&mut self, message: InstanceMessage) -> iced::Command<Message> {
        match message {
            InstanceMessage::Play(do_debug) => {
                let name1 = self.name.clone(); // (Jett voice)
                let name2 = self.name.clone(); // "Yikes!"

                iced::Command::batch(vec![
                    iced::Command::perform(dummy(), move |()| {
                        Message::InstanceMessage(
                            name1,
                            InstanceMessage::StateChanged(InstanceState::Playing),
                        )
                    }),
                    iced::Command::perform(
                        perform_play(
                            self.path.clone(),
                            self.executable.clone(),
                            self.name.clone(),
                            do_debug,
                        ),
                        move |()| {
                            Message::InstanceMessage(
                                name2,
                                InstanceMessage::StateChanged(InstanceState::Ready),
                            )
                        },
                    ),
                ])
            }
            InstanceMessage::Update => {
                let name = self.name.clone();
                iced::Command::batch(vec![
                    iced::Command::perform(dummy(), move |()| {
                        Message::InstanceMessage(
                            name,
                            InstanceMessage::StateChanged(InstanceState::Working(
                                "Updating".into(),
                            )),
                        )
                    }),
                    iced::Command::perform(perform_update(self.clone()), Message::Dummy),
                ])
            }
            InstanceMessage::Folder => {
                iced::Command::perform(open_folder(self.path.clone()), Message::Dummy)
            }
            InstanceMessage::Delete => {
                let name = self.name.clone();
                iced::Command::perform(delete(self.path.clone()), move |_| {
                    Message::RemoveInstance(Some(name))
                })
            }
            InstanceMessage::StateChanged(state) => {
                self.state = state;
                iced::Command::none()
            }
        }
    }

    pub fn view(&self) -> Element<InstanceMessage> {
        // Buttons
        let mut debug_button = Button::new(style::debug_icon()).style(icon_button());
        let mut play_button = Button::new(style::play_icon()).style(icon_button());
        let mut update_button = Button::new(style::update_icon()).style(icon_button());
        let folder_button = Button::new(style::reset_icon())
            .style(icon_button())
            .on_press(InstanceMessage::Folder);
        let mut delete_button = Button::new(style::delete_icon()).style(theme::Button::Destructive);

        if self.state.is_ready() {
            debug_button = debug_button.on_press(InstanceMessage::Play(true));
            play_button = play_button.on_press(InstanceMessage::Play(false));
            update_button = update_button.on_press(InstanceMessage::Update);
            delete_button = delete_button.on_press(InstanceMessage::Delete);
        }

        // Layout
        Row::new()
            .spacing(10)
            .padding(10)
            .align_items(Alignment::Start)
            .width(Length::Fill)
            .push(
                Column::new()
                    .push(Text::new(&self.name).size(24))
                    .push(Text::new(format!("Version: {:.*}", 32, self.version)).size(10))
                    .push(
                        Text::new(format!(
                            "Source: {} {}",
                            self.source.r#type, self.source.identifier
                        ))
                        .size(10),
                    ),
            )
            .push(Space::new(Length::Fill, Length::Shrink))
            .push({
                if let InstanceState::Working(progress) = &self.state {
                    let mut status_field = Column::new().align_items(Alignment::Center).push(
                        Text::new(&progress.status)
                            .size(16)
                            .horizontal_alignment(alignment::Horizontal::Center),
                    );
                    if let (Some(done), Some(total)) = (progress.done, progress.total) {
                        status_field = status_field.push(
                            ProgressBar::new(0.0..=total as f32, done as f32)
                                .height(Length::Fixed(5.)),
                        );
                    }
                    if let Some(done) = progress.done {
                        status_field = status_field.push(
                            Text::new(format!(
                                "{}/{}{}{}",
                                done,
                                if progress.total_approx { "~" } else { "" },
                                progress.total.map_or_else(|| "?".into(), |u| u.to_string()),
                                progress.units.as_ref().unwrap_or(&String::new())
                            ))
                            .size(12)
                            .horizontal_alignment(alignment::Horizontal::Center),
                        );
                    }
                    Row::new()
                        .push(Space::with_width(Length::FillPortion(1)))
                        .push(status_field.width(Length::FillPortion(2)))
                } else {
                    Row::new()
                        .spacing(10)
                        .push(debug_button)
                        .push(play_button)
                        .push(update_button)
                        .push(folder_button)
                        .push(delete_button)
                }
            })
            .into()
    }
}

async fn dummy() {}

pub async fn perform_install(
    path: PathBuf,
    name: String,
    instance_type: InstanceType,
    instance_source: InstanceSource,
) {
    send_message(Message::AddInstance(Box::new(Instance::new(
        path.clone(),
        "provisional".into(),
        name.clone(),
        instance_source.identifier.clone(),
        instance_type,
        instance_source.clone(),
        InstanceState::Working(Progress::default()),
    ))));
    match install::install(path, name.clone(), instance_type, instance_source) {
        Ok(instance) => {
            send_message(Message::AddInstance(Box::new(instance)));
        }
        Err(e) => {
            error!("Install failed: {:#}", e);
            send_message(Message::RemoveInstance(Some(name)));
        }
    }
}

pub async fn open_folder(path: PathBuf) {
    info!("Opening {} in file explorer", path.to_string_lossy());
    if let Err(e) = open::that(path.as_path()) {
        error!("Failed to open path: {}", e);
    }
}

pub async fn delete(path: PathBuf) -> Option<PathBuf> {
    if fs::remove_dir_all(&path).is_ok() {
        info!("Removed {}", path.to_string_lossy());
        Some(path)
    } else {
        error!("Failed to remove {}", path.to_string_lossy());
        None
    }
}

pub async fn perform_update(instance: Instance) {
    let name = instance.name.clone();
    match update::update_instance(instance).await {
        Ok(instance) => send_message(Message::AddInstance(Box::new(instance))),
        Err(e) => {
            error!("Failed to update instance: {:#}", e);
            send_message(Message::InstanceMessage(
                name,
                InstanceMessage::StateChanged(InstanceState::Ready),
            ));
        }
    }
}

pub async fn perform_play(path: PathBuf, executable: PathBuf, name: String, do_debug: bool) {
    send_message(Message::MusicMessage(MusicCommand::WeakPause));
    if let Err(e) = play(path, executable, name, do_debug).await {
        error!("Failed to run game: {:#}", e);
    }
    send_message(Message::MusicMessage(MusicCommand::WeakPlay));
}

pub async fn play(path: PathBuf, executable: PathBuf, name: String, do_debug: bool) -> Result<()> {
    let mut log_path = path;
    log_path.push("logs");
    fs::create_dir_all(&log_path)?;

    let time = OffsetDateTime::now_utc().format(&format_description::parse(
        "[year]-[month]-[day] [hour]-[minute]-[second]",
    )?)?;
    let mut out_path = log_path.clone();
    out_path.push(format!("{time}.out"));
    let mut out = File::create(out_path)?;

    let mut err_path = log_path.clone();
    err_path.push(format!("{time}.err"));
    let mut err = File::create(err_path)?;

    info!(
        "Launching {} via executable {}",
        name,
        executable.to_string_lossy()
    );

    let mut cmd = Command::new(&executable);
    let output = if do_debug {
        cmd.arg("-d").output()
    } else {
        cmd.output()
    };
    match output {
        Ok(output) => {
            info!("{} exited with {}", name, output.status);
            out.write_all(&output.stdout)?;
            err.write_all(&output.stderr)?;
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
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct InstancesContainer(Vec<Instance>);

pub fn perform_save_instances(instances: BTreeMap<String, Instance>) {
    if let Err(e) = save_instances(instances) {
        error!("Failed to save instances: {:#}", e);
    };
}

fn save_instances(instances: BTreeMap<String, Instance>) -> Result<()> {
    let mut instances_file =
        get_data_dir().ok_or_else(|| anyhow!("Failed to get Instances dir"))?;
    instances_file.push("instances.json");
    debug!("Saving to {}", instances_file.to_string_lossy());

    let file = File::create(instances_file)?;

    serde_json::to_writer_pretty(
        file,
        &InstancesContainer(instances.values().cloned().collect()),
    )?;
    Ok(())
}

pub fn load_instances() -> Result<Vec<Instance>> {
    let mut instances_file =
        get_data_dir().ok_or_else(|| anyhow!("Failed to get Instances dir"))?;
    instances_file.push("instances.json");
    debug!("Loading from {}", instances_file.to_string_lossy());

    if instances_file.exists() {
        let file = File::open(instances_file)?;

        let container: InstancesContainer = serde_json::from_reader(file)?;
        Ok(container.0)
    } else {
        warn!("instances.json doesn't exist (yet?), commencing without loading Instances");
        Ok(vec![])
    }
}
