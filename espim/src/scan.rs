use crate::{es_plugin_dir, InstalledPlugin};
use anyhow::Result;

/// Attempts to read plug-ins from the default directory
// One-line city, hell yeah.
pub fn scan_plugins() -> Result<Vec<InstalledPlugin>> {
    Ok(es_plugin_dir()
        .ok_or_else(|| anyhow!("Failed to get ES Plug-In dir"))?
        .read_dir()?
        .filter_map(|res| {
            Some(InstalledPlugin {
                name: String::from(res.ok()?.file_name().to_str()?),
            })
        })
        .collect())
}
