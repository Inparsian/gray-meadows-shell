use serde::{Deserialize, Serialize};

use super::{
    OpenAiServiceTier,
    OpenAiReasoningEffort,
    GeminiThinkingLevel,
    WeatherTemperatureUnit,
    WeatherSpeedUnit,
    WeatherPrecipitationUnit,
    AiService,
};

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
pub struct WeatherAlertsConfig {
    pub enabled: bool,
    pub refresh_interval: u64,
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
    pub alerts: WeatherAlertsConfig,
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