use serde::{Deserialize, Serialize};

use super::deserialize_insensitive;
use super::super::enums::{
    WeatherTemperatureUnit,
    WeatherSpeedUnit,
    WeatherPrecipitationUnit,
};

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