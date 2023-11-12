use crate::{get_data_dir, style, Message};
use anyhow::Context;
use anyhow::Result;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use espim::Plugin as EspimPlugin;
use iced::widget::{button, scrollable, Column, Container, Row, Scrollable, Space, Text};
use iced::{alignment, Alignment, Color, Command, Element, Length};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref CACHE_FILENAME_REGEX: Regex = Regex::new(r"[^\w.-]").unwrap();
}

#[derive(Debug, Clone)]
pub enum PluginsFrameState {
    Loading,
    Ready { plugins: Vec<Plugin> },
}

impl PluginsFrameState {
    pub fn new() -> (Self, Command<Message>) {
        (
            Self::Loading,
            Command::perform(load_plugins(), Message::PluginFrameLoaded),
        )
    }

    pub fn from(plugins: Vec<Plugin>) -> Self {
        Self::Ready { plugins }
    }

    pub fn view(&mut self) -> Container<Message> {
        match self {
            Self::Loading => Container::new(
                Column::new().align_items(Alignment::Center).push(
                    Text::new("Loading...")
                        .width(Length::Fill)
                        .color(Color::from_rgb(0.7, 0.7, 0.7))
                        .horizontal_alignment(alignment::Horizontal::Center),
                ),
            ),
            Self::Ready { plugins } => {
                let plugin_list = plugins.iter_mut().fold(
                    Column::new()
                        .padding(20)
                        .spacing(20)
                        .align_items(Alignment::Center),
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
                        .push(Scrollable::new(plugin_list))
                        .spacing(20)
                        .width(Length::Fill),
                )
                .width(Length::Fill)
                .padding(30)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum PluginMessage {
    Install,
    Remove,
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
            PluginMessage::Remove => {
                if let PluginState::Idle { espim_plugin } = &mut self.state {
                    espim_plugin.remove().unwrap_or_else(|e| {
                        error!("Failed to remove Plug-In {}: {}", self.name, e)
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
        if let Some(_bytes) = &self.icon_bytes {
            const ICON_DIMENSION: u16 = 48;
            content = content.push(
                Row::new()
                    .width(Length::Units(ICON_DIMENSION))
                    .align_items(Alignment::Center), // TODO: Re-enable when iced supports Image widgets with OpenGL
                                                     // .push(
                                                     //     Image::new(image::Handle::from_memory(bytes.clone())) // Not ideal, clones a couple KB every rendering pass
                                                     //         .height(Length::Units(ICON_DIMENSION))
                                                     //         .width(Length::Units(ICON_DIMENSION)),
                                                     // ),
            );
        }

        let mut infos = Column::new()
            .push(Text::new(&self.name).vertical_alignment(alignment::Vertical::Center));
        let mut controls = Row::new().spacing(10);

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

                let mut install_button = button::Button::new(style::update_icon()) // TODO: Use other icon here?
                    .style(style::Button::Icon);
                if espim_plugin.is_available() {
                    install_button = install_button.on_press(PluginMessage::Install)
                }

                let mut remove_button =
                    button::Button::new(style::delete_icon()).style(style::Button::Destructive);
                if espim_plugin.is_installed() {
                    remove_button = remove_button.on_press(PluginMessage::Remove)
                }

                controls = controls.push(install_button).push(remove_button);
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

pub async fn load_plugins() -> Vec<Plugin> {
    let mut plugins = vec![];
    match espim::retrieve_plugins() {
        Ok(retrieved) => {
            for p in retrieved {
                let name = String::from(p.name());
                let icon_bytes = load_icon_cached(&p)
                    .map_err(|e| debug!("failed to fetch icon: {}", e))
                    .ok();
                plugins.push(Plugin {
                    state: PluginState::Idle { espim_plugin: p },
                    name,
                    icon_bytes,
                })
            }
        }
        Err(e) => {
            error!(
                "Failed to initialize ESPIM, Plug-Ins will be unavailable: {:#}",
                e
            );
        }
    }
    plugins
}

fn get_cache_file(p: &EspimPlugin) -> Result<PathBuf> {
    let cache_dir = get_data_dir().unwrap().join("icons");
    if !(cache_dir.exists()) {
        std::fs::create_dir(&cache_dir).with_context(|| "Failed to create icon cache")?;
    }

    let version = p.versions().0.unwrap_or_else(|| p.versions().1.unwrap());
    let desired = format!("{}-{}", p.name(), version);
    let filename = CACHE_FILENAME_REGEX.replace_all(&desired, "_");
    Ok(cache_dir.join(&*filename))
}

fn load_icon_cached(p: &EspimPlugin) -> Result<Vec<u8>> {
    if p.is_installed() {
        return p
            .retrieve_icon()
            .ok_or_else(|| anyhow!("Failed to get item from installed plugin"));
    }

    let cache_file = get_cache_file(p)?;
    if cache_file.exists() && cache_file.is_file() && !p.is_installed() {
        let mut bytes = vec![];
        File::open(cache_file)?.read_to_end(&mut bytes)?;
        Ok(bytes)
    } else {
        let bytes = p
            .retrieve_icon()
            .with_context(|| "Failed to load icon from URL")?;
        File::create(cache_file)?.write_all(&bytes)?;
        Ok(bytes)
    }
}

pub async fn perform_install(mut plugin: EspimPlugin) -> EspimPlugin {
    if plugin.is_installed() {
        match get_cache_file(&plugin) {
            Ok(old_cache_file) => {
                if old_cache_file.exists() {
                    let _ = std::fs::remove_file(old_cache_file);
                }
            }
            Err(e) => {
                error!("Failed to get cache filename: {}", e)
            }
        };
    }

    if let Err(e) = plugin.download() {
        error!("Install failed: {:#}", e)
    }
    plugin
}
