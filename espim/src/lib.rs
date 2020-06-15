#![forbid(unsafe_code)]

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

mod index;
mod plugin;
mod scan;
mod util;

use anyhow::Result;
pub use plugin::Plugin;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
pub use util::unzip;

#[derive(Debug, Clone)]
struct InstalledPlugin {
    name: String,
    version: String,
}

impl InstalledPlugin {
    pub(crate) fn path(&self) -> PathBuf {
        let mut path = es_plugin_dir().unwrap();
        path.push(&self.name);
        path
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AvailablePlugin {
    name: String,
    url: String,
    version: String,
    #[serde(alias = "iconUrl")]
    icon_url: Option<String>,
    author: String,
    description: String,
}

pub fn retrieve_plugins() -> Result<Vec<Plugin>> {
    let available = index::get_available_plugins()?;
    let mut installed = scan::scan_plugins()?;
    let mut plugins = vec![];

    for a in available {
        let mut associated = None;
        for i in &mut installed {
            if i.name == a.name {
                associated = Some(i.clone());
                installed.retain(|p| p.name != associated.as_ref().unwrap().name);
                break;
            }
        }
        plugins.push(Plugin {
            installed: associated,
            available: Some(a),
        })
    }

    for i in installed {
        plugins.push(Plugin {
            installed: Some(i),
            available: None,
        })
    }

    Ok(plugins)
}

/// Get the Endless Sky Plug-In directory. On the 3 supported systems, this *should* never be None.
/// The path is not guaranteed to exist.
pub fn es_plugin_dir() -> Option<PathBuf> {
    Some(dirs::data_dir()?.join("endless-sky").join("plugins"))
}
