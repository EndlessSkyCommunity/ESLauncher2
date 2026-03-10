use crate::instance::{load_instances, Instance};
use crate::{Message, SharedSettings};
use iced::widget::{container, rule, Column, Container, Scrollable, Text};
use iced::{alignment, theme, Alignment, Color, Element, Length};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct InstancesFrame {
    pub instances: BTreeMap<String, Instance>,
}

impl InstancesFrame {
    pub fn new(settings: SharedSettings) -> Self {
        let instances = match load_instances(&settings.read().install_dir) {
            Ok(vec) => {
                let mut map = BTreeMap::new();
                for i in vec {
                    map.insert(i.name.clone(), i);
                }
                map
            }
            Err(e) => {
                error!("Failed to load instances: {:#}", e);
                BTreeMap::new()
            }
        };
        Self { instances }
    }

    pub fn view(&self) -> Element<Message> {
        let instances_column = Column::new()
            .padding(20)
            .spacing(5)
            .align_x(Alignment::Center);
        let instances_list: Element<_> = if self.instances.is_empty() {
            instances_column
                .push(
                    Text::new("No Instances yet")
                        .color(Color::from_rgb8(150, 150, 150))
                        .align_x(alignment::Horizontal::Center)
                        .width(Length::Fill),
                )
                .push(
                    Text::new("ESLauncher allows you to install multiple instances of Endless Sky. Instances are installations which ESLauncher automatically updates. Install your first instance by typing a name like 'newest' in the box to the right and choosing which version of the game to install.")
                        .size(16)
                         .color(Color::from_rgb8(150, 150, 150))
                        .align_x(alignment::Horizontal::Center)
                        .width(Length::Fill),
                )
                .into()
        } else {
            self.instances
                .values()
                .fold(instances_column, |column, instance| {
                    column
                        .push(
                            iced::widget::rule::horizontal(2).style(|theme: &iced::Theme| {
                                let mut style = rule::default(theme);
                                style.color.a *= 0.75;
                                style
                            }),
                        )
                        .push(instance.view().map(move |message| {
                            Message::InstanceMessage(instance.name.clone(), message)
                        }))
                })
                .into()
        };
        Container::new(Scrollable::new(
            Column::new()
                .push(
                    Text::new("Instances")
                        .size(26)
                        .align_x(alignment::Horizontal::Center)
                        .width(Length::Fill),
                )
                .push(instances_list)
                .spacing(20)
                .width(Length::Fill),
        ))
        .width(Length::FillPortion(3))
        .padding(30)
        .into()
    }
}
