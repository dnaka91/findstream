use std::{fs, time::Duration};

use anyhow::{Context, Result};
use serde::Deserialize;
use unidirs::{Directories, UnifiedDirs};

#[derive(Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub server: Server,
    pub twitch: Twitch,
    #[serde(default)]
    pub tracing: Tracing,
}

#[derive(Default, Deserialize)]
pub struct Server {
    #[serde(default)]
    pub concurrency_limit: Option<usize>,
    #[serde(default)]
    pub timeout: Option<Duration>,
}

#[derive(Deserialize)]
pub struct Twitch {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Default, Deserialize)]
pub struct Tracing {
    #[serde(default)]
    pub archer: Option<Archer>,
}

#[derive(Deserialize)]
pub struct Archer {
    pub address: String,
    pub certificate: String,
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
