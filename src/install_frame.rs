use crate::instance::{get_instances_dir, InstanceType};
use crate::{instance, Message};
use core::fmt;
use iced::{
    button, scrollable, text_input, Align, Button, Column, Command, Container, Element,
    HorizontalAlignment, Length, Radio, Scrollable, Text, TextInput,
};
use serde::{Deserialize, Serialize};

// Characters that shall not be allowed to enter. This does not cover all cases!
// One should expect the install process to fail on particularly exotic characters.
const BLACKLISTED_CHARS: [char; 10] = ['/', '\\', ':', '*', '?', '"', '<', '>', '|', '%'];

#[derive(Debug, Clone)]
pub struct InstallFrame {
    pub(crate) name: String,
    name_chooser: text_input::State,
    install_button: button::State,
    source: InstanceSource,
    source_identifier_input: text_input::State,
    scrollable: scrollable::State,
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
            identifier: String::from(""),
            r#type: InstanceSourceType::Continuous,
        }
    }
}

impl InstanceSourceType {
    pub const ALL: [Self; 3] = [Self::Continuous, Self::Release, Self::PR];
}

impl fmt::Display for InstanceSourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for InstallFrame {
    fn default() -> Self {
        Self {
            name: String::default(),
            name_chooser: text_input::State::default(),
            install_button: button::State::default(),
            source: InstanceSource::default(),
            source_identifier_input: text_input::State::default(),
            scrollable: scrollable::State::default(),
        }
    }
}

impl InstallFrame {
    pub fn update(&mut self, message: InstallFrameMessage) -> iced::Command<Message> {
        match message {
            InstallFrameMessage::StartInstallation(instance_type) => {
                if let Some(mut destination) = get_instances_dir() {
                    destination.push(&self.name);
                    return Command::perform(
                        instance::perform_install(
                            destination,
                            self.name.clone(),
                            instance_type,
                            self.source.clone(),
                        ),
                        Message::Installed,
                    );
                } else {
                    error!("Could not get instances directory from AppDirs")
                }
            }
            InstallFrameMessage::SourceTypeChanged(source_type) => self.source.r#type = source_type,
            InstallFrameMessage::NameChanged(name) => {
                if let Some(invalid) = name.chars().rfind(|c| BLACKLISTED_CHARS.contains(&c)) {
                    error!("Invalid character: '{}'", invalid)
                } else {
                    self.name = name
                }
            }
            InstallFrameMessage::SourceIdentifierChanged(identifier) => {
                self.source.identifier = identifier
            }
        }
        Command::none()
    }

    pub fn view(&mut self) -> Element<InstallFrameMessage> {
        let mut controls = InstanceSourceType::ALL.iter().fold(
            Column::new().spacing(10).push(Text::new("Choose a Type:")),
            |column, source_type| {
                column.push(Radio::new(
                    *source_type,
                    format!("{:?}", source_type),
                    Some(self.source.r#type),
                    InstallFrameMessage::SourceTypeChanged,
                ))
            },
        );
        if InstanceSourceType::Continuous != self.source.r#type {
            controls = controls.push(
                TextInput::new(
                    &mut self.source_identifier_input,
                    "Enter Version / Hash / PR Number",
                    &self.source.identifier,
                    InstallFrameMessage::SourceIdentifierChanged,
                )
                .padding(10),
            );
        }

        let mut install_button = Button::new(&mut self.install_button, Text::new("Install"));
        if !self.name.trim().is_empty() {
            install_button =
                install_button.on_press(InstallFrameMessage::StartInstallation(if cfg!(windows) {
                    InstanceType::Windows
                } else if cfg!(target_os = "linux") {
                    InstanceType::AppImage
                } else {
                    InstanceType::MacOS
                }))
        }

        Container::new(
            Scrollable::new(&mut self.scrollable).push(
                Column::new()
                    .padding(20)
                    .push(
                        Text::new("Install")
                            .horizontal_alignment(HorizontalAlignment::Center)
                            .width(Length::Fill)
                            .size(26),
                    )
                    .push(
                        TextInput::new(
                            &mut self.name_chooser,
                            "Name (required)",
                            &self.name,
                            InstallFrameMessage::NameChanged,
                        )
                        .padding(10),
                    )
                    .push(controls)
                    .push(install_button)
                    .spacing(20)
                    .align_items(Align::End),
            ),
        )
        .width(Length::FillPortion(2))
        .into()
    }
}
