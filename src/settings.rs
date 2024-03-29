use crate::get_data_dir;
use crate::music::MusicState;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Settings {
    pub music_state: MusicState,
}

impl Settings {
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
}
