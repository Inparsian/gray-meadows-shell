pub mod enums;

use std::sync::RwLock;
use std::{path::Path, sync::LazyLock};
use std::error::Error;
use notify::Watcher as _;
use notify::event::{AccessKind, AccessMode, EventKind};
use serde::{Deserialize, Serialize};

pub use enums::{
    OpenAiServiceTier,
    OpenAiReasoningEffort,
    GeminiThinkingLevel,
    WeatherTemperatureUnit,
    WeatherSpeedUnit,
    WeatherPrecipitationUnit,
    AiService,
};

use crate::utils::filesystem::get_config_directory;

static CONFIG: LazyLock<RwLock<Config>> = LazyLock::new(|| {
    let config = read().expect("Failed to read configuration");
    RwLock::new(config)
});

const FILE_NAME: &str = "config.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiFeatures {
    pub power_control: bool,
    pub mpris_control: bool,
    pub weather_info: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiConfig {
    pub api_key: String,
    pub model: String,
    #[serde(deserialize_with = "deserialize_insensitive")]
    pub service_tier: OpenAiServiceTier,
    #[serde(deserialize_with = "deserialize_insensitive")]
    pub reasoning_effort: OpenAiReasoningEffort,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiConfig {
    pub api_key: String,
    pub model: String,
    pub thinking_budget: i64,
    #[serde(deserialize_with = "deserialize_insensitive")]
    pub thinking_level: GeminiThinkingLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub enabled: bool,
    #[serde(deserialize_with = "deserialize_insensitive")]
    pub service: AiService,
    pub prompt: String,
    pub user_message_timestamps: bool,
    pub assistant_name: Option<String>,
    pub assistant_icon_path: Option<String>,
    pub openai: OpenAiConfig,
    pub gemini: GeminiConfig,
    pub features: AiFeatures,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherConfig {
    pub enabled: bool,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
    #[serde(deserialize_with = "deserialize_insensitive")]
    pub temperature_unit: WeatherTemperatureUnit,
    #[serde(deserialize_with = "deserialize_insensitive")]
    pub speed_unit: WeatherSpeedUnit,
    #[serde(deserialize_with = "deserialize_insensitive")]
    pub precipitation_unit: WeatherPrecipitationUnit,
    pub refresh_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ai: AiConfig,
    pub weather: WeatherConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            ai: AiConfig {
                enabled: true,
                service: AiService::OpenAi,
                prompt: "You are a helpful AI assistant running on a sidebar in a Linux desktop environment.".to_owned(),
                user_message_timestamps: true,
                assistant_name: None,
                assistant_icon_path: None,
                openai: OpenAiConfig {
                    api_key: "your-api-key-here".to_owned(),
                    model: "gpt-4.1".to_owned(),
                    service_tier: OpenAiServiceTier::Default,
                    reasoning_effort: OpenAiReasoningEffort::None,
                },
                gemini: GeminiConfig {
                    api_key: "your-api-key-here".to_owned(),
                    model: "gemini-2.0-flash".to_owned(),
                    thinking_budget: -1,
                    thinking_level: GeminiThinkingLevel::Budget,
                },
                features: AiFeatures {
                    power_control: true,
                    mpris_control: true,
                    weather_info: true,
                },
            },
            weather: WeatherConfig {
                enabled: true,
                latitude: 0.0,
                longitude: 0.0,
                timezone: "auto".to_owned(),
                temperature_unit: WeatherTemperatureUnit::Fahrenheit,
                speed_unit: WeatherSpeedUnit::Mph,
                precipitation_unit: WeatherPrecipitationUnit::Inch,
                refresh_interval: 1800,
            },
        }
    }
}

pub fn deserialize_insensitive<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(serde::de::Error::custom)
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
            info!(path = %config_path(), "Watching configuration file");

            for res in rx {
                match res {
                    Ok(event) => if event.paths.iter().any(|p| p.file_name() == Some("config.toml".as_ref()))
                        && matches!(event.kind, EventKind::Access(AccessKind::Close(AccessMode::Write)))
                    {
                        match read() {
                            Ok(new_config) => {
                                let mut config_lock = CONFIG.write().unwrap();
                                *config_lock = new_config;
                                info!("Configuration reloaded");
                            },
                            
                            Err(err) => {
                                warn!(%err, "Failed to reload configuration");
                            }
                        }
                    },

                    Err(e) => {
                        error!(%e, "Error watching configuration file");
                    },
                }
            }
        } else {
            error!(error = %result.unwrap_err(), "Failed to watch configuration file");
        }
    });
}

pub fn read_config() -> std::sync::RwLockReadGuard<'static, Config> {
    CONFIG.read().unwrap()
}

