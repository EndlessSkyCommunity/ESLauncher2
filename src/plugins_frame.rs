use crate::{style, Message};

use espim::Plugin as EspimPlugin;
use iced::{
    button, image, scrollable, Align, Color, Column, Command, Container, Element, Image, Length,
    Row, Scrollable, Space, Text, VerticalAlignment,
};

#[derive(Debug, Clone)]
pub struct PluginsFrameState {
    pub plugins: Vec<Plugin>,
    plugin_scrollable: scrollable::State,
}

impl PluginsFrameState {
    pub fn new() -> Self {
        let mut plugins = vec![];
        match espim::retrieve_plugins() {
            Ok(retrieved) => {
                for p in retrieved {
                    let name = String::from(p.name());
                    let icon_bytes = p.retrieve_icon();
                    plugins.push(Plugin {
                        state: PluginState::Idle { espim_plugin: p },
                        name,
                        icon_bytes,
                        install_button: button::State::default(),
                    })
                }
            }
            Err(e) => {
                error!(
                    "Failed to initialize ESPIM, Plug-Ins will unavailable: {}",
                    e
                );
            }
        }
        Self {
            plugins,
            plugin_scrollable: scrollable::State::default(),
        }
    }

    pub fn view(&mut self) -> Container<Message> {
        let plugin_list = self.plugins.iter_mut().fold(
            Column::new()
                .padding(20)
                .spacing(20)
                .align_items(Align::Center),
            |column, plugin| {
                let name = plugin.name.clone();
                column.push(
                    plugin
                        .view()
                        .map(move |msg| Message::PluginMessage(name.clone(), msg)),
                )
            },
        );

        Container::new(
            Column::new()
                .push(Scrollable::new(&mut self.plugin_scrollable).push(plugin_list))
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
    WorkFinished(EspimPlugin),
}

#[derive(Debug, Clone)]
pub enum PluginState {
    Working,
    Idle { espim_plugin: EspimPlugin },
}

#[derive(Debug, Clone)]
pub struct Plugin {
    pub state: PluginState,
    pub name: String,
    icon_bytes: Option<Vec<u8>>,
    install_button: button::State,
}

impl Plugin {
    pub fn update(&mut self, message: PluginMessage) -> Command<Message> {
        match message {
            PluginMessage::Install => {
                if let PluginState::Idle { espim_plugin } = &mut self.state {
                    let name = self.name.clone();
                    let plugin = espim_plugin.clone();
                    self.state = PluginState::Working;
                    return Command::perform(perform_install(plugin), move |p| {
                        Message::PluginMessage(name.clone(), PluginMessage::WorkFinished(p))
                    });
                }
            }
            PluginMessage::WorkFinished(plugin) => {
                self.state = PluginState::Idle {
                    espim_plugin: plugin,
                };
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<PluginMessage> {
        let mut content = Row::new().spacing(10).padding(10);
        if let Some(bytes) = &self.icon_bytes {
            content = content.push(
                Image::new(image::Handle::from_memory(bytes.clone())) // Not ideal, clones a couple KB every rendering pass
                    .height(Length::Units(48))
                    .width(Length::Units(48)),
            );
        }

        let mut infos =
            Column::new().push(Text::new(&self.name).vertical_alignment(VerticalAlignment::Center));
        let mut controls = Row::new();

        match &self.state {
            PluginState::Idle { espim_plugin } => {
                let versions = espim_plugin.versions();
                infos = infos
                    .push(
                        Text::new(if espim_plugin.is_installed() {
                            format!("Installed: {}", versions.0.unwrap_or("unknown"))
                        } else {
                            String::from("Not installed")
                        })
                        .size(14)
                        .color(Color::from_rgb(0.6, 0.6, 0.6)),
                    )
                    .push(
                        Text::new(if espim_plugin.is_available() {
                            format!("Available: {}", versions.1.unwrap_or("unknown"))
                        } else {
                            String::from("Unavailable")
                        })
                        .size(14)
                        .color(Color::from_rgb(0.6, 0.6, 0.6)),
                    );
                controls = controls.push(
                    button::Button::new(&mut self.install_button, style::update_icon()) //Use other icon here?
                        .style(style::Button::Icon)
                        .on_press(PluginMessage::Install),
                );
            }
            PluginState::Working => {
                infos = infos.push(
                    Text::new("Working...")
                        .size(14)
                        .color(Color::from_rgb(0.6, 0.6, 0.6)),
                )
            }
        };

        content
            .push(infos)
            .push(Space::new(Length::Fill, Length::Shrink))
            .push(controls)
            .into()
    }
}

pub async fn perform_install(mut plugin: EspimPlugin) -> EspimPlugin {
    if let Err(e) = plugin.install() {
        error!("Install failed: {}", e)
    }
    plugin
}
