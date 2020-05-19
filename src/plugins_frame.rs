use crate::Message;
use espim::ESPIM;
use iced::{image, Align, Column, Container, Element, Image, Length, Row, Text, VerticalAlignment};

#[derive(Debug, Clone)]
pub struct PluginsFrameState {
    espim: Option<ESPIM>,
    plugins: Vec<Plugin>,
}

impl PluginsFrameState {
    pub fn new() -> Self {
        match ESPIM::new() {
            Ok(espim) => Self {
                plugins: espim
                    .plugins
                    .iter()
                    .map(|p| Plugin {
                        espim_plugin: p.clone(),
                        icon_bytes: p.retrieve_icon(),
                    })
                    .collect(),
                espim: Some(espim),
            },
            Err(e) => {
                error!(
                    "Failed to initialize ESPIM, Plug-Ins will unavailable: {}",
                    e
                );
                Self {
                    espim: None,
                    plugins: vec![],
                }
            }
        }
    }
    pub fn view(&mut self) -> Container<Message> {
        let plugin_list = self.plugins.iter().fold(
            Column::new()
                .padding(20)
                .spacing(20)
                .align_items(Align::Center),
            |column, plugin| column.push(plugin.view()),
        );

        Container::new(
            Column::new()
                .push(plugin_list)
                .spacing(20)
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .padding(30)
    }
}

#[derive(Debug, Clone)]
struct Plugin {
    espim_plugin: espim::Plugin,
    icon_bytes: Option<Vec<u8>>,
}

impl Plugin {
    fn view(&self) -> Element<Message> {
        let mut content = Row::new().spacing(10).padding(10);
        if let Some(bytes) = &self.icon_bytes {
            content = content.push(
                Image::new(image::Handle::from_memory(bytes.clone())) // Not ideal, clones a couple KB every rendering pass
                    .height(Length::Units(64))
                    .width(Length::Units(64)),
            );
        }

        content
            .push(Text::new(self.espim_plugin.name()).vertical_alignment(VerticalAlignment::Center))
            .into()
    }
}
