use crate::{get_instances, Message};
use iced::{Align, Column, Container, Element, Text};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Instance {
    pub path: PathBuf,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct InstancesFrameState {
    pub instances: Vec<Instance>,
}

impl Default for InstancesFrameState {
    fn default() -> Self {
        InstancesFrameState {
            instances: get_instances(),
        }
    }
}

pub fn view(state: &mut InstancesFrameState) -> Element<Message> {
    let instances_list = state.instances.iter().fold(
        Column::new().padding(20).align_items(Align::Center),
        |column, instance| column.push(Text::new(&instance.name)),
    );

    Container::new(instances_list).into()
}
