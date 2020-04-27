use crate::instance::{get_instances_dir, InstanceType};
use crate::{instance, Message};
use iced::{
    button, text_input, Align, Button, Column, Command, Container, Element, HorizontalAlignment,
    Length, Radio, Text, TextInput,
};

#[derive(Debug, Clone)]
pub struct InstallFrame {
    pub(crate) name: String,
    name_chooser: text_input::State,
    install_button: button::State,
    source: InstanceSource,
    pr_chooser: text_input::State,
    pr_id: String,
}

#[derive(Debug, Clone)]
pub enum InstallFrameMessage {
    SourceChanged(InstanceSource),
    NameChanged(String),
    PrIdChanged(String),
    StartInstallation(InstanceType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceSource {
    Continuous,
    PR,
}
impl InstanceSource {
    pub const ALL: [InstanceSource; 2] = [InstanceSource::Continuous, InstanceSource::PR];
}

impl Default for InstallFrame {
    fn default() -> Self {
        InstallFrame {
            name: String::default(),
            name_chooser: text_input::State::default(),
            install_button: button::State::default(),
            source: InstanceSource::Continuous,
            pr_chooser: text_input::State::default(),
            pr_id: String::default(),
        }
    }
}

impl InstallFrame {
    pub fn update(&mut self, message: InstallFrameMessage) -> iced::Command<Message> {
        match message {
            InstallFrameMessage::StartInstallation(instance_type) => match get_instances_dir() {
                Some(mut destination) => {
                    destination.push(&self.name);
                    let name = String::from(destination.file_name().unwrap().to_string_lossy());
                    return Command::perform(
                        instance::perform_install(
                            destination,
                            name,
                            self.pr_id.clone(),
                            instance_type,
                            self.source,
                        ),
                        Message::Installed,
                    );
                }
                None => error!("Could not get instances directory from AppDirs"),
            },
            InstallFrameMessage::SourceChanged(source) => self.source = source,
            InstallFrameMessage::NameChanged(name) => self.name = name,
            InstallFrameMessage::PrIdChanged(pr_id) => self.pr_id = pr_id,
        }
        Command::none()
    }

    pub fn view(&mut self) -> Element<InstallFrameMessage> {
        let mut controls = InstanceSource::ALL.iter().fold(
            Column::new().spacing(10).push(Text::new("Choose a Type:")),
            |column, source| {
                column.push(Radio::new(
                    *source,
                    format!("{:?}", source),
                    Some(self.source),
                    InstallFrameMessage::SourceChanged,
                ))
            },
        );
        if let InstanceSource::PR = self.source {
            controls = controls.push(
                TextInput::new(
                    &mut self.pr_chooser,
                    "Enter PR ID",
                    &self.pr_id,
                    InstallFrameMessage::PrIdChanged,
                )
                .padding(10),
            );
        }

        let mut install_button = Button::new(&mut self.install_button, Text::new("Install"));
        if !self.name.is_empty() {
            install_button =
                install_button.on_press(InstallFrameMessage::StartInstallation(if cfg!(windows) {
                    InstanceType::Windows
                } else if cfg!(unix) {
                    InstanceType::AppImage
                } else {
                    InstanceType::MacOS
                }))
        }

        Container::new(
            Column::new()
                .padding(20)
                .max_width(400)
                .push(
                    Text::new("Install")
                        .horizontal_alignment(HorizontalAlignment::Center)
                        .width(Length::Fill)
                        .size(26),
                )
                .push(
                    TextInput::new(
                        &mut self.name_chooser,
                        "Choose Name",
                        &self.name,
                        InstallFrameMessage::NameChanged,
                    )
                    .padding(10),
                )
                .push(controls)
                .push(install_button)
                .spacing(20)
                .align_items(Align::End),
        )
        .into()
    }
}
