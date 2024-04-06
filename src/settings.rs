use crate::music::MusicState;
use crate::{get_data_dir, Message};
use anyhow::{Context, Result};
use iced::widget::{Checkbox, Column, Container, Row, Space, Text};
use iced::Length;
use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub music_state: MusicState,
    pub dark_theme: bool,
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    DarkTheme(bool),
}

impl Settings {
    fn default() -> Self {
        Self {
            music_state: MusicState::default(),
            dark_theme: dark_light::detect().eq(&dark_light::Mode::Dark),
        }
    }

    pub fn save(&self) -> Result<()> {
        let mut settings_file =
            get_data_dir().ok_or_else(|| anyhow!("Failed to get app save dir"))?;
        settings_file.push("settings.json");

        let file = File::create(settings_file)?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
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
        let settings_row = |label, content| {
            Row::new()
                .push(Text::new(label))
                .push(Space::with_width(Length::Fill))
                .push(content)
        };

        Container::new(
            Column::new().push(settings_row(
                "Dark Theme",
                Checkbox::new("", self.dark_theme)
                    .on_toggle(|v| Message::SettingsMessage(SettingsMessage::DarkTheme(v))),
            )),
        )
        .padding(100.)
    }

    pub fn update(&mut self, message: SettingsMessage) {
        match message {
            SettingsMessage::DarkTheme(bool) => self.dark_theme = bool,
        };
        if let Err(e) = self.save() {
            error!("Failed to save settings.json: {:#?}", e)
        };
    }
}
