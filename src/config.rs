use std::sync::RwLock;
use std::{path::Path, sync::LazyLock};
use std::error::Error;
use notify::Watcher as _;
use notify::event::{AccessKind, AccessMode, EventKind};
use serde::{Deserialize, Serialize};

use crate::filesystem::get_config_directory;

static CONFIG: LazyLock<RwLock<Config>> = LazyLock::new(|| {
    let config = read().expect("Failed to read configuration");
    RwLock::new(config)
});

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

fn save(config: &Config) -> Result<(), Box<dyn Error>> {
    std::fs::create_dir_all(get_config_directory())?;
    std::fs::write(
        config_path(),
        toml::to_string(config)?
    )?;

    Ok(())
}

fn read() -> Result<Config, Box<dyn Error>> {
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

pub fn watch() {
    tokio::spawn(async move {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher = notify::recommended_watcher(tx).unwrap();
        let result = watcher.watch(
            Path::new(&get_config_directory()),
            notify::RecursiveMode::NonRecursive,
        );

        if result.is_ok() {
            println!("Watching configuration file: {}", config_path());

            for res in rx {
                match res {
                    Ok(event) => if event.paths.iter().any(|p| p.file_name() == Some("config.toml".as_ref()))
                        && matches!(event.kind, EventKind::Access(AccessKind::Close(AccessMode::Write)))
                    {
                        if let Ok(new_config) = read() {
                            let mut config_lock = CONFIG.write().unwrap();
                            *config_lock = new_config;
                            println!("Configuration reloaded from {}", config_path());
                        } else {
                            eprintln!("Failed to reload configuration from {}", config_path());
                        }
                    },

                    Err(e) => {
                        eprintln!("Error watching configuration file: {}", e);
                    },
                }
            }
        } else {
            eprintln!("Failed to watch configuration file: {}", result.unwrap_err());
        }
    });
}

pub fn read_config() -> std::sync::RwLockReadGuard<'static, Config> {
    CONFIG.read().unwrap()
}

