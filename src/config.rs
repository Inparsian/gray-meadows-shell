use std::path::Path;
use std::error::Error;
use serde::{Deserialize, Serialize};

use crate::filesystem::get_config_directory;

const FILE_NAME: &str = "config.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiFeatures {
    pub power_control: bool,
    pub mpris_control: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiConfig {
    pub api_key: String,
    pub model: String,
    pub service_tier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiConfig {
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub enabled: bool,
    pub service: String,
    pub prompt: String,
    pub user_message_timestamps: bool,
    pub assistant_name: Option<String>,
    pub assistant_icon_path: Option<String>,
    pub openai: OpenAiConfig,
    pub gemini: GeminiConfig,
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
                service: "openai".to_owned(),
                prompt: "You are a helpful AI assistant running on a sidebar in a Linux desktop environment.".to_owned(),
                user_message_timestamps: true,
                assistant_name: None,
                assistant_icon_path: None,
                openai: OpenAiConfig {
                    api_key: "your-api-key-here".to_owned(),
                    model: "gpt-4.1".to_owned(),
                    service_tier: "default".to_owned(),
                },
                gemini: GeminiConfig {
                    api_key: "your-api-key-here".to_owned(),
                    model: "gemini-2.0-flash".to_owned(),
                },
                features: AiFeatures {
                    power_control: true,
                    mpris_control: true,
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

