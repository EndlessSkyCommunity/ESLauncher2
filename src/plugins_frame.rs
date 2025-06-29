use crate::style::icon_button;
use crate::{get_data_dir, style, Message};
use anyhow::Context;
use anyhow::Result;
use espim::Plugin as EspimPlugin;
use iced::widget::{button, image, rule, Column, Container, Image, Row, Scrollable, Space, Text};
use iced::{alignment, theme, Alignment, Color, Element, Length, Task};
use lazy_static::lazy_static;
use regex::Regex;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

lazy_static! {
    static ref CACHE_FILENAME_REGEX: Regex = Regex::new(r"[^\w.-]").unwrap();
}

#[derive(Debug, Clone)]
pub enum PluginsFrameState {
    Loading,
    Ready { plugins: Vec<Plugin> },
}

impl PluginsFrameState {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self::Loading,
            Task::perform(load_plugins(), Message::PluginFrameLoaded),
        )
    }

    pub fn from(plugins: Vec<Plugin>) -> Self {
        Self::Ready { plugins }
    }

    pub fn view(&self) -> Container<Message> {
        match self {
            Self::Loading => Container::new(
                Column::new().align_x(Alignment::Center).push(
                    Text::new("Loading...")
                        .width(Length::Fill)
                        .color(Color::from_rgb(0.7, 0.7, 0.7))
                        .align_x(alignment::Horizontal::Center),
                ),
            ),
            Self::Ready { plugins } => {
                let plugin_list =
                    plugins.iter().fold(
                        Column::new()
                            .padding(20)
                            .spacing(5)
                            .width(Length::Fill)
                            .align_x(Alignment::Center),
                        |column, plugin| {
                            column
                                .push(iced::widget::horizontal_rule(2).style(
                                    |theme: &iced::Theme| {
                                        let mut style = rule::default(theme);
                                        style.color.a *= 0.75;
                                        style
                                    },
                                ))
                                .push(plugin.view().map(move |msg| {
                                    Message::PluginMessage(plugin.name.clone(), msg)
                                }))
                        },
                    );

                Container::new(
                    Column::new()
                        .push(Scrollable::new(plugin_list))
                        .spacing(20)
                        .width(Length::Fill),
                )
                .width(Length::Fill)
                .padding(iced::Padding {
                    top: 0.0,
                    right: 30.0,
                    bottom: 0.0,
                    left: 30.0,
                })
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum PluginMessage {
    Install,
    Remove,
    OpenHREF,
    WorkFinished(Box<EspimPlugin>),
}

#[derive(Debug, Clone)]
pub enum PluginState {
    Working,
    Idle { espim_plugin: Box<EspimPlugin> },
}

#[derive(Debug, Clone)]
pub struct Plugin {
    pub state: PluginState,
    pub name: String,
    icon: Option<image::Handle>,
}

impl Plugin {
    pub fn update(&mut self, message: PluginMessage) -> Task<Message> {
        match message {
            PluginMessage::Install => {
                if let PluginState::Idle { espim_plugin } = &mut self.state {
                    let name = self.name.clone();
                    let plugin = espim_plugin.clone();
                    self.state = PluginState::Working;
                    return Task::perform(perform_install(*plugin), move |p| {
                        Message::PluginMessage(name, PluginMessage::WorkFinished(Box::new(p)))
                    });
                }
            }
            PluginMessage::Remove => {
                if let PluginState::Idle { espim_plugin } = &mut self.state {
                    espim_plugin.remove().unwrap_or_else(|e| {
                        error!("Failed to remove Plug-In {}: {}", self.name, e);
                    });
                }
            }
            PluginMessage::OpenHREF => {
                if let PluginState::Idle { espim_plugin } = &mut self.state {
                    if espim_plugin.is_available() {
                        let url = espim_plugin
                            .homepage()
                            .unwrap_or("No homepage available".to_string());
                        if url.trim().starts_with("http://") || url.trim().starts_with("https://") {
                            if open::that(url.trim()).is_err() {
                                error!("URL could not be opened: '{}'", url);
                            }
                        } else {
                            error!("URL validation failed: '{}' is not http(s):// .", url);
                        }
                    }
                }
            }
            PluginMessage::WorkFinished(plugin) => {
                self.state = PluginState::Idle {
                    espim_plugin: plugin,
                };
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<PluginMessage> {
        let content = Row::new().spacing(10).padding(10);
        const ICON_DIMENSION: f32 = 64.;
        let mut icon_container = Row::new()
            .width(Length::Fixed(ICON_DIMENSION))
            .align_y(Alignment::Center);
        if let Some(icon) = &self.icon {
            icon_container = icon_container.push(
                Image::new(icon.clone())
                    .height(Length::Fixed(ICON_DIMENSION))
                    .width(Length::Fixed(ICON_DIMENSION)),
            );
        }
        let mut textbox = Column::new().width(Length::Fill);
        let mut titlebox =
            Column::new().push(Text::new(&self.name).align_y(alignment::Vertical::Center));
        let mut infos = Column::new();

        let mut controls = Row::new().spacing(10);

        match &self.state {
            PluginState::Idle { espim_plugin } => {
                let versions = espim_plugin.versions();
                titlebox = titlebox
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
                infos = infos.push(Space::with_height(5)).push(
                    Text::new(
                        espim_plugin
                            .description()
                            .unwrap_or("Not available".to_string()),
                    )
                    .size(14)
                    .color(Color::from_rgb(0.6, 0.6, 0.6)),
                );

                let mut install_button =
                    button::Button::new(style::update_icon()).style(icon_button); // TODO: Use other icon here?
                if espim_plugin.is_available() {
                    install_button = install_button.on_press(PluginMessage::Install);
                }

                let mut remove_button = button::Button::new(style::delete_icon());
                // TODO .style(theme::Button::Destructive);
                if espim_plugin.is_installed() {
                    remove_button = remove_button.on_press(PluginMessage::Remove);
                }

                let mut href_button = button::Button::new(style::href_icon()).style(icon_button); // TODO: Use other icon here?
                if espim_plugin.is_available() {
                    href_button = href_button.on_press(PluginMessage::OpenHREF);
                }

                controls = controls
                    .push(href_button)
                    .push(install_button)
                    .push(remove_button);
            }
            PluginState::Working => {
                infos = infos.push(
                    Text::new("Working...")
                        .size(14)
                        .color(Color::from_rgb(0.6, 0.6, 0.6)),
                );
            }
        };
        let header = Row::new()
            .push(titlebox)
            .push(Space::new(Length::Fill, Length::Shrink))
            .push(controls);
        textbox = textbox.push(header).push(infos);

        content.push(icon_container).push(textbox).into()
    }
}

pub async fn load_plugins() -> Vec<Plugin> {
    let mut plugins = vec![];
    match espim::retrieve_plugins() {
        Ok(retrieved) => {
            for p in retrieved {
                let name = String::from(p.name());
                let icon = load_icon_cached(&p)
                    .map(image::Handle::from_memory)
                    .map_err(|e| debug!("failed to fetch icon: {}", e))
                    .ok();
                plugins.push(Plugin {
                    state: PluginState::Idle {
                        espim_plugin: Box::new(p),
                    },
                    name,
                    icon,
                });
            }
        }
        Err(e) => {
            error!(
                "Failed to initialize ESPIM, Plug-Ins will be unavailable: {:#}",
                e
            );
        }
    }
    plugins.sort_unstable_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
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
                error!("Failed to get cache filename: {}", e);
            }
        };
    }

    if let Err(e) = plugin.download() {
        error!("Install failed: {:#}", e);
    }
    plugin
}
