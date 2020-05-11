#![forbid(unsafe_code)]

#[macro_use]
extern crate anyhow;

mod scan;

use anyhow::Result;
use std::path::PathBuf;

pub struct InstalledPlugin {
    pub name: String,
}

pub struct ESPIM {
    installed_plugins: Vec<InstalledPlugin>,
}

impl ESPIM {
    pub fn new() -> Result<Self> {
        Ok(ESPIM {
            installed_plugins: scan::scan_plugins()?,
        })
    }

    /// A cached version of `scan_plugins()`
    pub fn installed_plugins(&self) -> &Vec<InstalledPlugin> {
        &self.installed_plugins
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
