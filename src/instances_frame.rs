use crate::instance::{load_instances, Instance};
use crate::Message;
use iced::{
    scrollable, Align, Color, Column, Container, Element, HorizontalAlignment, Length, Scrollable,
    Text,
};

#[derive(Debug, Clone)]
pub struct InstancesFrame {
    pub instances: Vec<Instance>,
    scrollable: scrollable::State,
}

impl Default for InstancesFrame {
    fn default() -> Self {
        let instances = match load_instances() {
            Ok(i) => i,
            Err(e) => {
                error!("Failed to load instances: {:#}", e);
                vec![]
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
            Text::new("No Instances yet")
                .color(Color::from_rgb8(150, 150, 150))
                .horizontal_alignment(HorizontalAlignment::Center)
                .width(Length::Fill)
                .into()
        } else {
            self.instances
                .iter_mut()
                .enumerate()
                .fold(instances_column, |column, (i, instance)| {
                    column.push(
                        instance
                            .view()
                            .map(move |message| Message::InstanceMessage(i, message)),
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
