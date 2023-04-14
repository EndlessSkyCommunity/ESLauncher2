use iced::{Text, Element, Column, Button, button, Length, Alignment, alignment, Row, text_input, Padding};

use crate::{Message, instance::Instance, style};

#[derive(Debug, Clone)]
pub enum AdvancedMessage {
    NameTextInputChanged(String),
    ArgsTextInputChanged(String),
}

#[derive(Debug)]
pub struct AdvancedFrame {
    old_name: String,
    instance: Option<Instance>,

    close_button: button::State,
    name_input: text_input::State,
    args_input: text_input::State
}

impl Default for AdvancedFrame {
    fn default() -> Self {
        Self {
            old_name: String::default(),
            instance: None,
            close_button: button::State::default(),
            name_input: text_input::State::default(),
            args_input: text_input::State::default()
        }
    }
}

impl AdvancedFrame {
    pub fn new(new_instance: Instance) -> Self{
        Self {
            old_name: String::from(&new_instance.name),
            instance: Some(new_instance),
            ..Default::default()
        }
    }

    pub fn update(&mut self, message: AdvancedMessage) {
        match message {
            AdvancedMessage::NameTextInputChanged(string) => {
                self.instance.as_mut().unwrap().name = string;
            }

            AdvancedMessage::ArgsTextInputChanged(string) => {
                self.instance.as_mut().unwrap().args = string;
            }
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        
        let close_button = Button::new(&mut self.close_button, style::close_icon())
        .style(style::Button::Icon)
        .on_press(match &self.instance {
            Some(inst) => Message::CloseAdvanced(self.old_name.clone(), inst.clone()),
            None => Message::Dummy(()),
        });

        let mut p = Padding::new(0);
        p.bottom = 60;

        let out = Column::new()
        .push(
        Row::new()
                .width(Length::Fill)
                .align_items(Alignment::End) 
                .push(
                    Text::new("Advanced Settings")
                        .size(26)
                        .horizontal_alignment(alignment::Horizontal::Center)
                        .width(Length::Fill),
                )
                .push(close_button)
                .padding(p)
        )
        .padding(40)
        .width(iced::Length::FillPortion(3))
        .spacing(6);

        if let Some(instance) = &self.instance {

            let name_input = text_input::TextInput::new(
                &mut self.name_input,
                "Instance name",
                &instance.name,
                name_text_input_changed
            ).padding(4);

            let out = out
            .push(
                Row::new()
                .push(
                    {
                        let text = format!("Instance name:");
                        Text::new(text)
                        .size(18)
                    }
                )
                .push(
                    name_input
                )
                .spacing(14)
                .align_items(Alignment::Center)
            )
            .push(
                text_input::TextInput::new(
                    &mut self.args_input,
                    "Executable arguments",
                    &instance.args,
                    args_text_input_changed
                ).padding(4)
            );

            out
            .push(
                Text::new("-h or --help to log help")
            )
            .into()
        } else {
            out
            .push(Text::new("How in the..."))
            .into()
        }
    }
}

fn name_text_input_changed(string: String) -> Message {
    Message::AdvancedMessage(AdvancedMessage::NameTextInputChanged(string))
}

fn args_text_input_changed(string: String) -> Message {
    Message::AdvancedMessage(AdvancedMessage::ArgsTextInputChanged(string))
}
