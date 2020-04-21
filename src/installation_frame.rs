use crate::Message;
use iced::{button, Align, Button, Column, Container, Element, Row, Text};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct InstallationFrameState {
    pub destination: PathBuf,
    pub destination_chooser: button::State,
    pub install_button: button::State,
}

impl Default for InstallationFrameState {
    fn default() -> Self {
        InstallationFrameState {
            destination: PathBuf::default(),
            destination_chooser: button::State::default(),
            install_button: button::State::default(),
        }
    }
}

pub fn view(state: &mut InstallationFrameState) -> Element<Message> {
    let mut install_button = Button::new(&mut state.install_button, Text::new("Install"));
    if !state.destination.eq(&PathBuf::default()) {
        install_button = install_button.on_press(Message::StartInstallation)
    }

    Container::new(
        Column::new()
            .push(
                Row::new()
                    .padding(20)
                    .align_items(Align::Center)
                    .push(Text::new(state.destination.to_string_lossy()))
                    .push(
                        Button::new(&mut state.destination_chooser, Text::new("Pick Folder"))
                            .on_press(Message::SelectDestination),
                    ),
            )
            .push(install_button),
    )
    .into()
}
