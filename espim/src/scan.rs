use crate::{es_plugin_dir, InstalledPlugin};
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

/// Attempts to read plug-ins from the default directory
pub(crate) fn scan_plugins() -> Result<Vec<InstalledPlugin>> {
    let plugin_dir = es_plugin_dir().ok_or_else(|| anyhow!("Failed to get ES Plug-In dir"))?;
    Ok(if plugin_dir.exists() {
        plugin_dir
            .read_dir()?
            .filter_map(|res| {
                let res = res.ok()?;
                let version = read_version(&res.path()).unwrap_or_else(|_| String::from("unknown"));
                Some(InstalledPlugin {
                    name: String::from(res.file_name().to_str()?),
                    version,
                })
            })
            .collect()
    } else {
        vec![]
    })
}

fn read_version(plugin_dir: &PathBuf) -> std::io::Result<String> {
    let mut path = plugin_dir.clone();
    path.push(".version");
    fs::read_to_string(path)
}
