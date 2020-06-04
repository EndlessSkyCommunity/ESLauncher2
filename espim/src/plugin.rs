use crate::{util, AvailablePlugin, InstalledPlugin};
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

/// Possible icon names, in order of preference
const ICON_NAMES: [&str; 4] = ["icon@2x.png", "icon@2x.jpg", "icon.png", "icon.jpg"];

#[derive(Debug, Clone)]
pub struct Plugin {
    pub(crate) installed: Option<InstalledPlugin>,
    pub(crate) available: Option<AvailablePlugin>,
}

impl Plugin {
    pub fn is_installed(&self) -> bool {
        self.installed.is_some()
    }

    pub fn is_available(&self) -> bool {
        self.available.is_some()
    }

    pub fn name(&self) -> &str {
        if let Some(i) = &self.installed {
            &i.name
        } else if let Some(a) = &self.available {
            &a.name
        } else {
            panic!("Plugin that is neither installed nor available")
        }
    }

    /// Returns the plug-in's path, if it is installed
    pub fn path(&self) -> Option<PathBuf> {
        Some(self.installed.as_ref()?.path())
    }

    /// Attempts to retrieve the plug-in's icon from a number of sources.
    pub fn retrieve_icon(&self) -> Option<Vec<u8>> {
        if let Some(i) = &self.installed {
            for name in ICON_NAMES.iter() {
                let mut path = i.path();
                path.push(name);
                if path.exists() {
                    return std::fs::read(path).ok();
                }
            }
        }
        if let Some(a) = &self.available {
            return util::download(a.icon_url.as_ref()?).ok();
        }
        None
    }

    /// Downloads & installs the plug-in to `es_plugin_folder()`/`name()`
    pub fn download(&mut self) -> Result<()> {
        self.installed = Some(
            self.available
                .as_ref()
                .ok_or_else(|| anyhow!("Not an available Plug-In"))?
                .download()?,
        );
        Ok(())
    }

    /// Removes the plug-in locally
    pub fn remove(&mut self) -> Result<()> {
        fs::remove_dir_all(
            self.path()
                .ok_or_else(|| anyhow!("Not an installed Plug-In"))?,
        )?;
        self.installed = None;
        Ok(())
    }

    /// Returns the (`installed`, `available`) versions, if known
    pub fn versions(&self) -> (Option<&str>, Option<&str>) {
        (
            self.installed.as_ref().map(|i| i.version.as_str()),
            self.available.as_ref().map(|a| a.version.as_str()),
        )
    }
}
