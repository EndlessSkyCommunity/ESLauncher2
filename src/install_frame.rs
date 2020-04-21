use crate::Message;
use iced::{button, text_input, Align, Button, Column, Container, Element, Row, Text, TextInput};

#[derive(Debug, Clone)]
pub struct InstallFrameState {
    pub name: String,
    pub name_chooser: text_input::State,
    pub install_button: button::State,
}

impl Default for InstallFrameState {
    fn default() -> Self {
        InstallFrameState {
            name: String::default(),
            name_chooser: text_input::State::default(),
            install_button: button::State::default(),
        }
    }
}

pub fn view(state: &mut InstallFrameState) -> Element<Message> {
    let mut install_button = Button::new(&mut state.install_button, Text::new("Install"));
    if !state.name.is_empty() {
        install_button = install_button.on_press(Message::StartInstallation)
    }

    Container::new(
        Column::new()
            .push(
                Row::new()
                    .padding(20)
                    .align_items(Align::Center)
                    .push(Text::new("Name:"))
                    .push(TextInput::new(
                        &mut state.name_chooser,
                        "Choose Name",
                        &state.name,
                        Message::NameChanged,
                    )),
            )
            .push(install_button),
    )
    .into()
}
