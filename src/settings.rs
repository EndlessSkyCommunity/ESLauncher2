use crate::music::MusicState;
use crate::{get_data_dir, style, Message};
use anyhow::{Context, Result};
use iced::advanced::graphics::core::Element;
use iced::widget::text_input::StyleSheet;
use iced::widget::{checkbox, container, row, text, Text};
use iced::{
    widget::{button, Column, Container, Row},
    Length,
};
use iced::{Alignment, Command, Padding, Renderer};
use serde::{Deserialize, Serialize};
use std::{fs::File, path::PathBuf};

#[derive(Clone, Debug)]
pub enum CustomInstallPath {
    SetEnabled(bool),
    RequestPath,
    SetPath(PathBuf),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub music_state: MusicState,
    pub dark_theme: bool,
    pub custom_install_dir: Option<PathBuf>,
    pub use_custom_install_dir: bool,
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    DarkTheme(bool),
    CustomInstallPath(CustomInstallPath),
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

        match File::open(settings_file)
            .with_context(|| "Failed to open settings.json")
            .and_then(|f| {
                serde_json::from_reader(f).with_context(|| "Failed to deserialize settings.json")
            }) {
            Ok(s) => s,
            Err(e) => {
                warn!("{:#?}", e);
                Self::default()
            }
        }
    }

    pub fn view(&self) -> Container<Message> {
        fn settings_row<'a>(
            label: &'a str,
            content: impl Into<Element<'a, Message, iced::Theme, Renderer>>,
            enabled: bool,
        ) -> impl Into<Element<'a, Message, iced::Theme, Renderer>> {
            let setting_spacer = || {
                iced::widget::horizontal_rule(2).style(iced::theme::Rule::from(
                    |theme: &iced::Theme| {
                        let mut appearance =
                            iced::widget::rule::StyleSheet::appearance(theme, &Default::default());
                        appearance.color.a *= 0.75;
                        appearance
                    },
                ))
            };
            let container = container(
                Column::new()
                    .push(setting_spacer())
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
        let btn = button(style::folder_icon().size(12.0))
            .on_press_maybe(
                self.use_custom_install_dir
                    .then_some(Message::SettingsMessage(
                        SettingsMessage::CustomInstallPath(CustomInstallPath::RequestPath),
                    )),
            )
            // .style(icon_button())
            .padding(Padding::from([2, 0]));
        Container::new(
            Column::new()
                .push(settings_row(
                    "Use custom install directory",
                    checkbox("", self.use_custom_install_dir).on_toggle(|f| {
                        Message::SettingsMessage(SettingsMessage::CustomInstallPath(
                            CustomInstallPath::SetEnabled(f),
                        ))
                    }),
                    true,
                ))
                .push(settings_row(
                    "Custom install directory",
                    row!(
                        text(format!(
                            "Installing to {}",
                            self.custom_install_dir
                                .clone()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .as_ref()
                        ))
                        .size(12.0),
                        btn,
                    )
                    .align_items(Alignment::Center)
                    .spacing(10.0)
                    .padding(Padding {
                        top: 0.0,
                        right: 10.0,
                        bottom: 0.0,
                        left: 0.0,
                    }),
                    self.use_custom_install_dir,
                ))
                .spacing(10.0),
        )
        .padding(100.0)
    }

    pub fn update(&mut self, message: SettingsMessage) -> Command<Message> {
        match message {
            SettingsMessage::CustomInstallPath(custom_install_path) => match custom_install_path {
                CustomInstallPath::RequestPath => {
                    return Command::perform(
                        rfd::AsyncFileDialog::new().pick_folder(),
                        |f| match f {
                            Some(handle) => {
                                Message::SettingsMessage(SettingsMessage::CustomInstallPath(
                                    CustomInstallPath::SetPath(handle.path().to_path_buf()),
                                ))
                            }
                            None => Message::Dummy(()),
                        },
                    )
                }
                CustomInstallPath::SetEnabled(f) => {
                    self.use_custom_install_dir = f;
                }
                CustomInstallPath::SetPath(p) => {
                    self.custom_install_dir = Some(p);
                }
            },
        };
        self.save();

        Command::none()
    }
}
