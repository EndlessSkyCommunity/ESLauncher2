use crate::{style, Message};

use espim::ESPIM;
use iced::{
    button, image, Align, Color, Column, Command, Container, Element, Image, Length, Row, Space,
    Text, VerticalAlignment,
};

#[derive(Debug, Clone)]
pub struct PluginsFrameState {
    espim: Option<ESPIM>,
    pub plugins: Vec<Plugin>,
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
                        install_button: button::State::default(),
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
        let plugin_list = self.plugins.iter_mut().fold(
            Column::new()
                .padding(20)
                .spacing(20)
                .align_items(Align::Center),
            |column, plugin| {
                let name = String::from(plugin.espim_plugin.name());
                column.push(
                    plugin
                        .view()
                        .map(move |msg| Message::PluginMessage(name.clone(), msg)),
                )
            },
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
pub enum PluginMessage {
    Install,
}

#[derive(Debug, Clone)]
pub struct Plugin {
    pub espim_plugin: espim::Plugin,
    icon_bytes: Option<Vec<u8>>,
    install_button: button::State,
}

impl Plugin {
    pub fn update(&mut self, message: PluginMessage) -> Command<Message> {
        match message {
            PluginMessage::Install => {
                Command::perform(perform_install(self.espim_plugin.clone()), Message::Dummy)
            }
        }
    }

    fn view(&mut self) -> Element<PluginMessage> {
        let versions = self.espim_plugin.versions();
        let mut content = Row::new().spacing(10).padding(10);
        if let Some(bytes) = &self.icon_bytes {
            content = content.push(
                Image::new(image::Handle::from_memory(bytes.clone())) // Not ideal, clones a couple KB every rendering pass
                    .height(Length::Units(48))
                    .width(Length::Units(48)),
            );
        }

        content
            .push(
                Column::new()
                    .push(
                        Text::new(self.espim_plugin.name())
                            .vertical_alignment(VerticalAlignment::Center),
                    )
                    .push(
                        Text::new(if self.espim_plugin.is_installed() {
                            format!("Installed: {}", versions.0.unwrap_or("unknown"))
                        } else {
                            String::from("Not installed")
                        })
                        .size(14)
                        .color(Color::from_rgb(0.6, 0.6, 0.6)),
                    )
                    .push(
                        Text::new(if self.espim_plugin.is_available() {
                            format!("Available: {}", versions.1.unwrap_or("unknown"))
                        } else {
                            String::from("Unavailable")
                        })
                        .size(14)
                        .color(Color::from_rgb(0.6, 0.6, 0.6)),
                    ),
            )
            .push(Space::new(Length::Fill, Length::Shrink))
            .push(
                button::Button::new(&mut self.install_button, style::update_icon()) //Use other icon here?
                    .style(style::Button::Icon)
                    .on_press(PluginMessage::Install),
            )
            .into()
    }
}

pub async fn perform_install(mut plugin: espim::Plugin) {
    match plugin.install() {
        Ok(_) => {}
        Err(e) => error!("Install failed: {}", e),
    }
}
