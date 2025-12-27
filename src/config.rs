use std::path::Path;
use std::error::Error;
use serde::{Deserialize, Serialize};

use crate::filesystem::get_config_directory;

const FILE_NAME: &str = "config.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiFeatures {
    pub current_date_time: bool,
    pub power_control: bool,
    pub mpris_control: bool,
    pub wayland_info: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub enabled: bool,
    pub api_key: String,
    pub model: String,
    pub service_tier: String,
    pub prompt: String,
    pub features: AiFeatures,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ai: AiConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            ai: AiConfig {
                enabled: true,
                api_key: "your-api-key-here".to_owned(),
                model: "gpt-4.1".to_owned(),
                service_tier: "default".to_owned(),
                prompt: "You are a helpful AI assistant running on a sidebar in a Linux desktop environment.".to_owned(),
                features: AiFeatures {
                    current_date_time: true,
                    power_control: true,
                    mpris_control: true,
                    wayland_info: true,
                },
            },
        }
    }
}

fn config_path() -> String {
    format!("{}/{}", get_config_directory(), FILE_NAME)
}

pub fn save(config: &Config) -> Result<(), Box<dyn Error>> {
    std::fs::create_dir_all(get_config_directory())?;
    std::fs::write(
        config_path(),
        toml::to_string(config)?
    )?;

    Ok(())
}

pub fn read() -> Result<Config, Box<dyn Error>> {
    if !Path::new(&config_path()).exists() {
        let default = Config::default();
        save(&default)?;
        Ok(default)
    } else {
        let toml = std::fs::read_to_string(config_path())?;
        let config = toml::from_str(&toml)?;
        Ok(config)
    }
}

