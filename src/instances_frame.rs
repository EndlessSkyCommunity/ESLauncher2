use crate::instance::{load_instances, Instance};
use crate::Message;
use iced::{
    scrollable, Align, Color, Column, Container, Element, HorizontalAlignment, Length, Scrollable,
    Text,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct InstancesFrame {
    pub instances: BTreeMap<String, Instance>,
    scrollable: scrollable::State,
}

impl Default for InstancesFrame {
    fn default() -> Self {
        let instances = match load_instances() {
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
        Self {
            instances,
            scrollable: scrollable::State::default(),
        }
    }
}
impl InstancesFrame {
    pub fn view(&mut self) -> Element<Message> {
        let instances_column = Column::new()
            .padding(20)
            .spacing(20)
            .align_items(Align::Center);
        let instances_list: Element<_> = if self.instances.is_empty() {
            instances_column
                .push(
                    Text::new("No Instances yet")
                        .color(Color::from_rgb8(150, 150, 150))
                        .horizontal_alignment(HorizontalAlignment::Center)
                        .width(Length::Fill),
                )
                .push(
                    Text::new("ESLauncher allows you to install multiple instances of Endless Sky. Instances are installations which ESLauncher automatically updates. Install your first instance by typing a name like 'newest' in the box to the right and choosing which version of the game to install.")
                        .size(16)
                        .color(Color::from_rgb8(150, 150, 150))
                        .horizontal_alignment(HorizontalAlignment::Center)
                        .width(Length::Fill),
                )
                .into()
        } else {
            self.instances
                .values_mut()
                .fold(instances_column, |column, instance| {
                    let name = instance.name.clone();
                    column.push(
                        instance
                            .view()
                            .map(move |message| Message::InstanceMessage(name.clone(), message)),
                    )
                })
                .into()
        };
        Container::new(
            Scrollable::new(&mut self.scrollable).push(
                Column::new()
                    .push(
                        Text::new("Instances")
                            .size(26)
                            .horizontal_alignment(HorizontalAlignment::Center)
                            .width(Length::Fill),
                    )
                    .push(instances_list)
                    .spacing(20)
                    .width(Length::Fill),
            ),
        )
        .width(Length::FillPortion(3))
        .padding(30)
        .into()
    }
}
