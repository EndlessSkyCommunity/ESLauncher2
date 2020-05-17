#![forbid(unsafe_code)]

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

mod index;
mod scan;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct InstalledPlugin {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AvailablePlugin {
    pub name: String,
    pub url: String,
    pub version: String,
    #[serde(alias = "iconUrl")]
    pub icon_url: String,
    pub author: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct ESPIM {
    available_plugins: Vec<AvailablePlugin>,
    installed_plugins: Vec<InstalledPlugin>,
}

impl ESPIM {
    pub fn new() -> Result<Self> {
        Ok(ESPIM {
            available_plugins: index::get_available_plugins()?,
            installed_plugins: scan::scan_plugins()?,
        })
    }

    /// Scans installed plug-ins, returning them and refreshing ESPIM's cache
    pub fn retrieve_installed_plugins(&mut self) -> Result<&Vec<InstalledPlugin>> {
        self.installed_plugins = scan::scan_plugins()?;
        Ok(&self.installed_plugins)
    }

    /// A cached version of `retrieve_installed_plugins()`
    pub fn installed_plugins(&self) -> &Vec<InstalledPlugin> {
        &self.installed_plugins
    }

    /// Retrieves available plug-ins, returning them and refreshing ESPIM's cache
    pub fn retrieve_available_plugins(&mut self) -> Result<&Vec<AvailablePlugin>> {
        self.available_plugins = index::get_available_plugins()?;
        Ok(&self.available_plugins)
    }

    /// A cached version of `retrieve_available_plugins()`
    pub fn available_plugins(&self) -> &Vec<AvailablePlugin> {
        &self.available_plugins
    }
}

/// Get the Endless Sky Plug-In directory. On the 3 supported systems, this *should* never be None.
/// The path is not guaranteed to exist.
pub fn es_plugin_dir() -> Option<PathBuf> {
    Some(dirs::data_dir()?.join("endless-sky").join("plugins"))
}

#[cfg(test)]
mod tests {
    use crate::{es_plugin_dir, AvailablePlugin, ESPIM};
    use std::fs;

    #[test]
    fn initialize() {
        ESPIM::new().unwrap();
    }

    #[test]
    fn download_wf() {
        let wf = AvailablePlugin {
            name: String::from( "World Forge"),
            url: String::from("https://github.com/EndlessSkyCommunity/world-forge"),
            version: String::from("22f036fcff384dcdd41c583783597eb994b9ab7a"),
            icon_url: String::from("https://github.com/EndlessSkyCommunity/world-forge/raw/master/icon.png"),
            author: String::from("Amazinite"),
            description: String::from("A plugin for Endless Sky that allows the player to access everything in the game in one place. Includes features that the all-content plugin does not have such as the ability to boost your combat rating and change your friendly/hostile status with factions of the game without having to save-edit. Intended to help content creators test their plugins."),
        };
        let mut out = es_plugin_dir().unwrap();
        out.push(&wf.name);
        wf.download().unwrap();
        fs::remove_dir_all(out).unwrap();
    }
}
