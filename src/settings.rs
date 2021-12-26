use std::fs;

use anyhow::{Context, Result};
use serde::Deserialize;
use unidirs::{Directories, UnifiedDirs};

#[derive(Deserialize)]
pub struct Settings {
    pub client_id: String,
    pub client_secret: String,
}

pub fn load() -> Result<Settings> {
    let path = UnifiedDirs::simple("rocks", "dnaka91", env!("CARGO_PKG_NAME"))
        .default()
        .context("failed finding project directories")?
        .config_dir()
        .join("config.toml");

    let buf = fs::read(path).context("failed reading settings file")?;

    toml::from_slice(&buf).context("failed parsing settings")
}
