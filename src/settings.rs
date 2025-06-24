use crate::music::MusicState;
use crate::{get_data_dir, style, Message};
use anyhow::{Context, Result};
use iced::advanced::graphics::core::Element;
use iced::widget::text_input::StyleSheet;
use iced::widget::{container, row, text, Checkbox, Text};
use iced::{
    widget::{button, Column, Container, Row},
    Length,
};
use iced::{Alignment, Command, Padding, Renderer};
use serde::{Deserialize, Serialize};
use std::{fs::File, path::PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub music_state: MusicState,
    pub dark_theme: bool,
    #[serde(default = "default_install_dir")]
    pub install_dir: PathBuf,
}
fn default_install_dir() -> PathBuf {
    get_data_dir().unwrap().join("instances")
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    DarkTheme(bool),
    RequestInstallPath,
    SetInstallPath(PathBuf),
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            music_state: Default::default(),
            dark_theme: dark_light::detect().eq(&dark_light::Mode::Dark),
            install_dir: default_install_dir(),
        }
    }
}

impl Settings {
    pub fn save(&self) {
        let save = || -> Result<()> {
            let mut settings_file =
                get_data_dir().ok_or_else(|| anyhow!("Failed to get app save dir"))?;
            settings_file.push("settings.json");

            let file = File::create(settings_file)?;
            serde_json::to_writer_pretty(file, self)?;
            Ok(())
        };
        if let Err(e) = save() {
            error!("Failed to save settings.json: {:#?}", e);
        }
    }

    pub fn load() -> Self {
        let mut settings_file = get_data_dir()
            .ok_or_else(|| anyhow!("Failed to get app save dir"))
            .unwrap();
        settings_file.push("settings.json");

        if !settings_file.exists() {
            return Self::default();
        }

        File::open(settings_file)
            .with_context(|| "Failed to open settings.json")
            .and_then(|f| {
                serde_json::from_reader(f).with_context(|| "Failed to deserialize settings.json")
            })
            .unwrap_or_else(|e| {
                warn!("{:#?}", e);
                Self::default()
            })
    }

    pub fn view(&self) -> Container<Message> {
        fn settings_row<'a>(
            label: &'a str,
            content: impl Into<Element<'a, Message, iced::Theme, Renderer>>,
            enabled: bool,
        ) -> impl Into<Element<'a, Message, iced::Theme, Renderer>> {
            let container = container(
                Column::new()
                    .push(
                        Row::new()
                            .push(Text::new(label))
                            .push(
                                container(content)
                                    .align_x(iced::alignment::Horizontal::Right)
                                    .width(Length::Fill),
                            )
                            .align_items(Alignment::Center),
                    )
                    .spacing(10.0),
            );
            if enabled {
                container
            } else {
                container.style(iced::theme::Container::Custom(Box::new(
                    |theme: &iced::Theme| container::Appearance {
                        text_color: Some(
                            theme.disabled(&iced::theme::TextInput::Default).icon_color,
                        ),
                        ..Default::default()
                    },
                )))
            }
        }

        let install_dir_picker = button(style::folder_icon().size(12.0))
            .on_press(Message::SettingsMessage(
                SettingsMessage::RequestInstallPath,
            ))
            // .style(icon_button())
            .padding(Padding::from([2, 0]));
        let install_dir_reset_btn = if self.install_dir.eq(&default_install_dir()) {
            None
        } else {
            Some(
                button(style::reset_icon().size(12.0))
                    .on_press(Message::SettingsMessage(SettingsMessage::SetInstallPath(
                        default_install_dir(),
                    )))
                    .padding(Padding::from([2, 0])),
            )
        };

        Container::new(
            Column::new()
                .push(settings_row(
                    "Install directory",
                    row!(
                        text(format!(
                            "Installing to {}",
                            self.install_dir.to_string_lossy(),
                        ))
                        .size(12.0),
                        install_dir_picker
                    )
                    .push_maybe(install_dir_reset_btn)
                    .align_items(Alignment::Center)
                    .spacing(10.0)
                    .padding(Padding {
                        top: 0.0,
                        right: 10.0,
                        bottom: 0.0,
                        left: 0.0,
                    }),
                    true,
                ))
                .push(settings_row(
                    "Dark Theme",
                    Checkbox::new("", self.dark_theme)
                        .on_toggle(|v| Message::SettingsMessage(SettingsMessage::DarkTheme(v))),
                    true,
                ))
                .spacing(10.0),
        )
        .padding(100.0)
    }

    pub fn update(&mut self, message: SettingsMessage) -> Command<Message> {
        match message {
            SettingsMessage::RequestInstallPath => {
                return Command::perform(rfd::AsyncFileDialog::new().pick_folder(), |f| match f {
                    Some(handle) => Message::SettingsMessage(SettingsMessage::SetInstallPath(
                        handle.path().to_path_buf(),
                    )),
                    None => Message::Dummy(()),
                })
            }
            SettingsMessage::SetInstallPath(p) => {
                self.install_dir = p;
            }
            SettingsMessage::DarkTheme(dark) => self.dark_theme = dark,
        };
        self.save();

        Command::none()
    }
}
