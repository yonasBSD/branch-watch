use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub token: Option<String>,
}

fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".branch-watch.toml")
}

pub fn load() -> Result<Config> {
    let path = config_path();
    if !path.exists() {
        return Ok(Config::default());
    }
    let contents = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config: {}", path.display()))?;
    toml::from_str(&contents).context("Failed to parse config")
}

pub fn save(config: &Config) -> Result<()> {
    let path = config_path();
    let contents = toml::to_string_pretty(config).context("Failed to serialize config")?;
    fs::write(&path, contents)
        .with_context(|| format!("Failed to write config: {}", path.display()))?;
    println!("Token saved to {}", path.display());
    Ok(())
}
