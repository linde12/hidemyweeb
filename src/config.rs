// use anyhow;
// use dirs;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub wallpaper_directory: PathBuf,
    pub wallpaper_whitelist: Vec<PathBuf>,
}

impl Config {
    pub fn new() -> anyhow::Result<Self> {
        let config_path = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
            .join("hidemyweeb")
            .join("config.toml");

        let config_content = std::fs::read_to_string(&config_path)
            .map_err(|e| anyhow::anyhow!("Failed to read config file: {}", e))?;

        let config: Config = toml::from_str(&config_content)
            .map_err(|e| anyhow::anyhow!("Failed to parse config file: {}", e))?;

        Ok(config)
    }
}
