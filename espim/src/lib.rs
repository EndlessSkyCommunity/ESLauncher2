#![forbid(unsafe_code)]

#[macro_use]
extern crate anyhow;

mod index;
mod scan;

use crate::index::AvailablePlugin;
use anyhow::Result;
use std::path::PathBuf;

pub struct InstalledPlugin {
    pub name: String,
}

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
    use crate::ESPIM;

    #[test]
    fn initialize() {
        ESPIM::new().unwrap();
    }
}
