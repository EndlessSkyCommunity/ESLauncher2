use crate::instances::{get_instances, Instance};
use crate::Message;
use iced::{Align, Color, Column, Container, Element, Row, Text};

#[derive(Debug, Clone)]
pub struct InstancesFrameState {
    pub instances: Vec<Instance>,
}

impl Default for InstancesFrameState {
    fn default() -> Self {
        InstancesFrameState {
            instances: get_instances().unwrap_or_else(|| vec![]),
        }
    }
}

pub fn view(state: &mut InstancesFrameState) -> Element<Message> {
    let instances_column = Column::new().padding(20).align_items(Align::Center);
    let instances_list = if state.instances.is_empty() {
        instances_column.push(Text::new("No Instances yet").color(Color::from_rgb8(150, 150, 150)))
    } else {
        state
            .instances
            .iter()
            .fold(instances_column, |column, instance| {
                column.push(Row::new().push(Text::new(&instance.name)))
            })
    };

    Container::new(
        Column::new()
            .push(Text::new("Instances:"))
            .push(instances_list)
            .spacing(20),
    )
    .padding(30)
    .into()
}
