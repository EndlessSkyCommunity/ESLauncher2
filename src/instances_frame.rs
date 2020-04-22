use crate::instances::{get_instances, Instance};
use crate::Message;
use iced::{Align, Color, Column, Container, Element, Text};

#[derive(Debug, Clone)]
pub struct InstancesFrameState {
    pub instances: Vec<Instance>,
}

impl Default for InstancesFrameState {
    fn default() -> Self {
        let instances = get_instances().unwrap_or_else(|| vec![]);
        InstancesFrameState { instances }
    }
}

pub fn view(state: &mut InstancesFrameState) -> Element<Message> {
    let instances_column = Column::new()
        .padding(20)
        .spacing(20)
        .align_items(Align::Center);
    let instances_list: Element<_> = if state.instances.is_empty() {
        Text::new("No Instances yet")
            .color(Color::from_rgb8(150, 150, 150))
            .into()
    } else {
        state
            .instances
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
        Column::new()
            .push(Text::new("Instances").size(26))
            .push(instances_list)
            .spacing(20),
    )
    .padding(30)
    .into()
}
