use std::fs;

use anyhow::{bail, Result};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub client_id: String,
    pub client_secret: String,
}

pub fn load() -> Result<Settings> {
    let locations = &["/app/findstream.toml", "findstream.toml"];
    let buf = locations.iter().find_map(|loc| fs::read(loc).ok());

    match buf {
        Some(buf) => Ok(toml::from_slice(&buf)?),
        None => bail!("failed finding settings"),
    }
}
