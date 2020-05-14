use anyhow::Result;
use attohttpc;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct PluginIndex(Vec<AvailablePlugin>);

#[derive(Serialize, Deserialize)]
pub struct AvailablePlugin {
    name: String,
    url: String,
    version: String,
    #[serde(alias = "iconUrl")]
    icon_url: String,
    author: String,
    description: String,
}

pub fn get_available_plugins() -> Result<Vec<AvailablePlugin>> {
    let resp = attohttpc::get(
        "https://raw.githubusercontent.com/MCOfficer/endless-sky-plugins/master/plugins.json",
    )
    .send()?;
    let index: PluginIndex = resp.json()?;
    Ok(index.0)
}
