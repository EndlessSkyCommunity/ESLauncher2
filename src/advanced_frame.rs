use iced::{Text, Element, Column, Button, button, Length, Alignment, alignment, Row, text_input, Padding};

use crate::{Message, instance::Instance, style};

#[derive(Debug)]
pub struct AdvancedFrame {
    old_name: String,
    instance: Option<Instance>,

    close_button: button::State,
    _name_input: text_input::State
}

impl Default for AdvancedFrame {
    fn default() -> Self {
        Self {
            old_name: String::default(),
            instance: None,
            close_button: button::State::default(),
            _name_input: text_input::State::default()
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
        .padding(30)
        .width(iced::Length::FillPortion(3));

        if let Some(instance) = &self.instance {
            out
            .push(
                {
                    let text = format!("Instance name: {}", &instance.name);
                    Text::new(text)
                    .size(18)
                }
            )
            .into()
        } else {
            out
            .push(Text::new("How in the..."))
            .into()
        }
        
    }
}
