use crate::Message;
use espim::{AvailablePlugin, InstalledPlugin, ESPIM};
use iced::{Align, Column, Container, Element, Length, Row, Text};

#[derive(Debug, Clone)]
pub struct PluginsFrameState {
    espim: Option<ESPIM>,
    plugins: Vec<Plugin>,
}

impl PluginsFrameState {
    pub fn new() -> Self {
        match ESPIM::new() {
            Ok(espim) => {
                let mut plugins: Vec<Plugin> = espim
                    .installed_plugins()
                    .iter()
                    .map(|p| Plugin::Installed { plugin: p.clone() })
                    .collect();
                for p in espim.available_plugins() {
                    if !espim.installed_plugins().iter().any(|i| i.name.eq(&p.name)) {
                        plugins.push(Plugin::Available { plugin: p.clone() });
                    }
                }
                Self {
                    espim: Some(espim),
                    plugins,
                }
            }
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
enum Plugin {
    Available { plugin: AvailablePlugin },
    Installed { plugin: InstalledPlugin },
}

impl Plugin {
    fn view(&self) -> Element<Message> {
        let content = Row::new().spacing(10).padding(10);
        match self {
            Plugin::Available { plugin } => {
                content.push(Text::new(format!("Available: {}", plugin.name.clone())))
            }
            Plugin::Installed { plugin } => {
                content.push(Text::new(format!("Installed: {}", plugin.name.clone())))
            }
        }
        .into()
    }
}
