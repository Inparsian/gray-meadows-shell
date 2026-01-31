// ! This is not comprehensive; only the fields we care about are included.
// ! If more fields need to be added, they'll be added here.
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenMeteoResponseCurrentUnits {
    pub time: String,
    pub interval: String,
    pub is_day: String,
    pub temperature_2m: String,
    pub relative_humidity_2m: String,
    pub apparent_temperature: String,
    pub precipitation: String,
    pub rain: String,
    pub showers: String,
    pub snowfall: String,
    pub weather_code: String,
    pub cloud_cover: String,
    pub pressure_msl: String,
    pub surface_pressure: String,
    pub wind_speed_10m: String,
    pub wind_direction_10m: String,
    pub wind_gusts_10m: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenMeteoResponseCurrent {
    pub time: String,
    pub interval: i64,
    pub is_day: i64,
    pub temperature_2m: f64,
    pub relative_humidity_2m: f64,
    pub apparent_temperature: f64,
    pub precipitation: f64,
    pub rain: f64,
    pub showers: f64,
    pub snowfall: f64,
    pub weather_code: i64,
    pub cloud_cover: f64,
    pub pressure_msl: f64,
    pub surface_pressure: f64,
    pub wind_speed_10m: f64,
    pub wind_direction_10m: f64,
    pub wind_gusts_10m: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenMeteoResponseDailyUnits {
    pub time: String,
    pub weather_code: String,
    pub temperature_2m_max: String,
    pub temperature_2m_min: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenMeteoResponseDaily {
    pub time: Vec<String>,
    pub weather_code: Vec<i64>,
    pub temperature_2m_max: Vec<f64>,
    pub temperature_2m_min: Vec<f64>,
}

// A helper struct, not part of the API response.
#[derive(Debug, Clone)]
pub struct OpenMeteoResponseDailyItem {
    pub time: String,
    pub weather_code: i64,
    pub temperature_2m_max: f64,
    pub temperature_2m_min: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenMeteoResponse {
    pub latitude: f64,
    pub longitude: f64,
    pub generationtime_ms: f64,
    pub utc_offset_seconds: i64,
    pub timezone: String,
    pub timezone_abbreviation: String,
    pub elevation: f64,
    pub current_units: OpenMeteoResponseCurrentUnits,
    pub current: OpenMeteoResponseCurrent,
    pub daily_units: OpenMeteoResponseDailyUnits,
    pub daily: OpenMeteoResponseDaily,
}