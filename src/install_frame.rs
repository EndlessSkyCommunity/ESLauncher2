use crate::instance::InstanceType;
use crate::settings::Settings;
use crate::style::text_button;
use crate::{instance, Message};
use core::fmt;
use iced::widget::{Button, Column, Container, Radio, Scrollable, Text, TextInput};
use iced::{alignment, Alignment, Element, Length, Task};
use serde::{Deserialize, Serialize};

// Characters that shall not be allowed to enter. This does not cover all cases!
// One should expect the install process to fail on particularly exotic characters.
const BLACKLISTED_CHARS: [char; 10] = ['/', '\\', ':', '*', '?', '"', '<', '>', '|', '%'];

#[derive(Debug, Clone, Default)]
pub struct InstallFrame {
    pub(crate) name: String,
    source: InstanceSource,
}

#[derive(Debug, Clone)]
pub enum InstallFrameMessage {
    SourceTypeChanged(InstanceSourceType),
    NameChanged(String),
    SourceIdentifierChanged(String),
    StartInstallation(InstanceType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstanceSourceType {
    Release,
    Continuous,
    PR,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceSource {
    pub(crate) identifier: String,
    pub(crate) r#type: InstanceSourceType,
}

impl Default for InstanceSource {
    fn default() -> Self {
        Self {
            identifier: String::new(),
            r#type: InstanceSourceType::Continuous,
        }
    }
}

impl InstanceSourceType {
    pub const ALL: [Self; 3] = [Self::Continuous, Self::Release, Self::PR];
}

impl fmt::Display for InstanceSourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl InstallFrame {
    pub fn update(
        &mut self,
        message: InstallFrameMessage,
        settings: &mut Settings,
    ) -> Task<Message> {
        match message {
            InstallFrameMessage::StartInstallation(instance_type) => {
                return Task::perform(
                    instance::perform_install(
                        settings.install_dir.join(&self.name),
                        self.name.clone(),
                        instance_type,
                        self.source.clone(),
                    ),
                    Message::Dummy,
                );
            }
            InstallFrameMessage::SourceTypeChanged(source_type) => self.source.r#type = source_type,
            InstallFrameMessage::NameChanged(name) => {
                if let Some(invalid) = name.chars().rfind(|c| BLACKLISTED_CHARS.contains(c)) {
                    error!("Invalid character: '{}'", invalid);
                } else {
                    self.name = name;
                }
            }
            InstallFrameMessage::SourceIdentifierChanged(identifier) => {
                self.source.identifier = identifier;
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<InstallFrameMessage> {
        let mut controls = InstanceSourceType::ALL.iter().fold(
            Column::new().spacing(10).push(Text::new("Choose a Type:")),
            |column, source_type| {
                column.push(Radio::new(
                    format!("{source_type:?}"),
                    *source_type,
                    Some(self.source.r#type),
                    InstallFrameMessage::SourceTypeChanged,
                ))
            },
        );
        if InstanceSourceType::Continuous != self.source.r#type {
            controls = controls.push(
                TextInput::new("Enter Version / Hash / PR Number", &self.source.identifier)
                    .on_input(InstallFrameMessage::SourceIdentifierChanged)
                    .padding(10),
            );
        }

        let mut install_button = Button::new(Text::new("Install")).style(text_button);
        if !self.name.trim().is_empty() {
            install_button =
                install_button.on_press(InstallFrameMessage::StartInstallation(if cfg!(windows) {
                    InstanceType::Windows
                } else if cfg!(target_os = "linux") {
                    InstanceType::AppImage
                } else {
                    InstanceType::MacOS
                }));
        }

        Container::new(Scrollable::new(
            Column::new()
                .padding(20)
                .push(
                    Text::new("Install")
                        .align_x(alignment::Horizontal::Center)
                        .width(Length::Fill)
                        .size(26),
                )
                .push(
                    TextInput::new("Name (required)", &self.name)
                        .on_input(InstallFrameMessage::NameChanged)
                        .padding(10),
                )
                .push(controls)
                .push(install_button)
                .spacing(20)
                .align_x(Alignment::End),
        ))
        .width(Length::FillPortion(2))
        .into()
    }
}
