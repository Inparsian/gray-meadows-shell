pub mod schemas {
    pub mod openmeteo;
    pub mod nws;
}

use std::sync::LazyLock;
use futures_signals::signal::Mutable;
use reqwest::Client;

use crate::config::read_config;
use crate::sql::wrappers::weather::{get_weather_forecast, set_weather_forecast, get_weather_alerts, set_weather_alerts};
use self::schemas::openmeteo::{OpenMeteoResponse, OpenMeteoResponseDailyItem};
use self::schemas::nws::{NwsAlertsResponse, NwsAlertsError};

pub static WEATHER: LazyLock<Weather> = LazyLock::new(Weather::default);

const OPENMETEO_API_URL: &str = "https://api.open-meteo.com/v1/forecast";
const NWS_ACTIVE_ALERTS_API_URL: &str = "https://api.weather.gov/alerts/active";

pub struct WmoCode {
    pub text: &'static str,
    pub short_text: &'static str,
    pub day_icon: &'static str,
    pub night_icon: &'static str,
}

impl WmoCode {
    pub fn new(
        text: &'static str,
        short_text: &'static str,
        day_icon: &'static str,
        night_icon: &'static str,
    ) -> Self {
        Self {
            text,
            short_text,
            day_icon,
            night_icon,
        }
    }

    pub fn get_icon(&self, is_day: bool) -> &'static str {
        if is_day {
            self.day_icon
        } else {
            self.night_icon
        }
    }
}

#[derive(Clone, Debug)]
pub struct Weather {
    pub client: Client,
    pub last_response: Mutable<Option<OpenMeteoResponse>>,
    pub last_alerts_response: Mutable<Option<NwsAlertsResponse>>,
}

impl Default for Weather {
    fn default() -> Self {
        Self {
            client: Client::builder()
                .user_agent("GrayMeadowsShell/1.0")
                .build()
                .unwrap_or_default(),
            last_response: Mutable::new(None),
            last_alerts_response: Mutable::new(None),
        }
    }
}

impl Weather {
    /// Fetches the latest weather data from the OpenMeteo API and updates the cache.
    pub async fn fetch(&self) {
        let weather_config = read_config().weather.clone();

        let current = [
            "is_day",
            "temperature_2m",
            "relative_humidity_2m",
            "apparent_temperature",
            "precipitation",
            "rain",
            "showers",
            "snowfall",
            "weather_code",
            "cloud_cover",
            "pressure_msl",
            "surface_pressure",
            "wind_speed_10m",
            "wind_direction_10m",
            "wind_gusts_10m",
        ];

        let daily = [
            "weather_code",
            "temperature_2m_max",
            "temperature_2m_min",
        ];
        
        let parameters = [
            ("latitude", weather_config.latitude.to_string()),
            ("longitude", weather_config.longitude.to_string()),
            ("timezone", weather_config.timezone.clone()),
            ("temperature_unit", weather_config.temperature_unit.to_string()),
            ("wind_speed_unit", weather_config.speed_unit.to_string()),
            ("precipitation_unit", weather_config.precipitation_unit.to_string()),
            ("forecast_days", "7".to_owned()),
            ("current", current.join(",")),
            ("daily", daily.join(",")),
        ];

        match self.client.get(OPENMETEO_API_URL)
            .query(&parameters)
            .send()
            .await
        {
            Ok(response) => match response.json::<OpenMeteoResponse>().await {
                Ok(weather_data) => {
                    let _ = set_weather_forecast(&weather_data).await;
                    self.last_response.set(Some(weather_data));
                }

                Err(err) => {
                    error!(?err, "Failed to parse weather data");
                }
            },

            Err(err) => {
                error!(?err, "Failed to fetch weather data");
            },
        }
    }
    
    /// Fetches the latest alerts from the National Weather Service.
    pub async fn fetch_nws_alerts(&self) -> Option<NwsAlertsError> {
        let weather_config = read_config().weather.clone();
        
        match self.client.get(NWS_ACTIVE_ALERTS_API_URL)
            .query(&[("point", format!("{},{}", weather_config.latitude, weather_config.longitude))])
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => match response.json::<NwsAlertsResponse>().await {
                Ok(alerts) => {
                    let _ = set_weather_alerts(&alerts).await;
                    self.last_alerts_response.set(Some(alerts));
                    None
                },

                Err(err) => {
                    error!(?err, "Failed to parse NWS alerts");
                    None
                },
            },
            
            Ok(response) => match response.json::<NwsAlertsError>().await {
                Ok(err) => {
                    error!(?err, "Failed to fetch NWS alerts");
                    Some(err)
                },
                
                Err(err) => {
                    error!(?err, "Failed to parse NWS alerts error");
                    None
                },
            },

            Err(err) => {
                error!(?err, "Failed to fetch NWS alerts");
                None
            },
        }
    }

    /// Returns the last cached forecast's age in seconds, if there was a hit.
    pub async fn cache_check(&self) -> Option<i64> {
        if let Ok((fetched_at, forecast)) = get_weather_forecast().await {
            let now = chrono::Utc::now().naive_utc();
            let elapsed = now.signed_duration_since(fetched_at).num_seconds();
            debug!(elapsed, "Weather cache hit");
            self.last_response.set(Some(forecast));
            Some(elapsed)
        } else {
            None
        }
    }
    
    /// Returns the last cached weather alerts response's age in seconds, if there was a hit.
    pub async fn cache_check_alerts(&self) -> Option<i64> {
        if let Ok((fetched_at, alerts)) = get_weather_alerts().await {
            let now = chrono::Utc::now().naive_utc();
            let elapsed = now.signed_duration_since(fetched_at).num_seconds();
            debug!(elapsed, "Weather alerts cache hit");
            self.last_alerts_response.set(Some(alerts));
            Some(elapsed)
        } else {
            None
        }
    }
}

pub fn get_daily_at(forecast: &OpenMeteoResponse, i: usize) -> Option<OpenMeteoResponseDailyItem> {
    (i < forecast.daily.time.len()).then(|| OpenMeteoResponseDailyItem {
        time: forecast.daily.time[i].clone(),
        weather_code: forecast.daily.weather_code[i],
        temperature_2m_max: forecast.daily.temperature_2m_max[i],
        temperature_2m_min: forecast.daily.temperature_2m_min[i],
    })
}

pub fn get_wmo_code(code: i64) -> Option<WmoCode> {
    let wmo = match code {
        0 => WmoCode::new("Clear sky", "clear", "clear_day", "moon_stars"),
        1 => WmoCode::new("Mainly clear", "clearish", "partly_cloudy_day", "partly_cloudy_night"),
        2 => WmoCode::new("Partly cloudy", "cloudyish", "partly_cloudy_day", "partly_cloudy_night"),
        3 => WmoCode::new("Mostly cloudy", "cloudy", "cloud", "cloud"),
        45 | 48 => WmoCode::new("Fog", "fog", "foggy", "foggy"),
        51 => WmoCode::new("Light drizzle", "drizzle (l)", "rainy_light", "rainy_light"),
        53 => WmoCode::new("Drizzle", "drizzle", "rainy", "rainy"),
        55 => WmoCode::new("Dense drizzle", "drizzle (h)", "rainy_heavy", "rainy_heavy"),
        56 => WmoCode::new("Light freezing drizzle", "drizzle (fl)", "rainy_light", "rainy_light"),
        57 => WmoCode::new("Dense freezing drizzle", "drizzle (fh)", "rainy_heavy", "rainy_heavy"),
        61 => WmoCode::new("Slight rain", "rain (l)", "rainy_light", "rainy_light"),
        63 => WmoCode::new("Rain", "rain", "rainy", "rainy"),
        65 => WmoCode::new("Heavy rain", "rain (h)", "rainy_heavy", "rainy_heavy"),
        66 => WmoCode::new("Light freezing rain", "rain (fl)", "rainy_light", "rainy_light"),
        67 => WmoCode::new("Heavy freezing rain", "rain (fh)", "rainy_heavy", "rainy_heavy"),
        71 => WmoCode::new("Slight snow", "snow (l)", "weather_snowy", "weather_snowy"),
        73 => WmoCode::new("Snow", "snow", "weather_snowy", "weather_snowy"),
        75 => WmoCode::new("Heavy snow", "snow (h)", "weather_snowy", "weather_snowy"),
        77 => WmoCode::new("Snow grains", "snow grains", "weather_snowy", "weather_snowy"),
        80 => WmoCode::new("Light rain showers", "showers (l)", "rainy_light", "rainy_light"),
        81 => WmoCode::new("Rain showers", "showers", "rainy", "rainy"),
        82 => WmoCode::new("Heavy rain showers", "showers (h)", "rainy_heavy", "rainy_heavy"),
        85 => WmoCode::new("Light snow showers", "snow showers (l)", "weather_snowy", "weather_snowy"),
        86 => WmoCode::new("Heavy snow showers", "snow showers (h)", "weather_snowy", "weather_snowy"),
        95 => WmoCode::new("Thunderstorm", "storm", "thunderstorm", "thunderstorm"),
        96 => WmoCode::new("Thunderstorm, slight hail", "storm (lH)", "thunderstorm", "thunderstorm"),
        99 => WmoCode::new("Thunderstorm, heavy hail", "storm (hH)", "thunderstorm", "thunderstorm"),
        _ => return None,
    };
    
    Some(wmo)
}

pub fn activate() {
    // OpenMeteo task
    tokio::spawn(async move {
        let weather_config = {
            let config = read_config();
            config.weather.clone()
        };

        if !weather_config.enabled {
            return;
        }

        let clamped_interval = weather_config.refresh_interval.max(600);
        if let Some(elapsed) = WEATHER.cache_check().await && elapsed < clamped_interval as i64 {
            let sleep_duration = clamped_interval as i64 - elapsed;
            info!(sleep_duration, "Weather cache valid, sleeping until next refresh");
            tokio::time::sleep(std::time::Duration::from_secs(sleep_duration as u64)).await;
        }

        loop {
            WEATHER.fetch().await;
            tokio::time::sleep(std::time::Duration::from_secs(clamped_interval)).await;
        }
    });
    
    // NWS task
    tokio::spawn(async move {
        let weather_config = {
            let config = read_config();
            config.weather.clone()
        };
        
        if !weather_config.enabled || !weather_config.alerts.enabled {
            return;
        }
        
        let clamped_interval = weather_config.alerts.refresh_interval.max(10);
        if let Some(elapsed) = WEATHER.cache_check_alerts().await && elapsed < clamped_interval as i64 {
            let sleep_duration = clamped_interval as i64 - elapsed;
            info!(sleep_duration, "Weather alerts cache valid, sleeping until next refresh");
            tokio::time::sleep(std::time::Duration::from_secs(sleep_duration as u64)).await;
        }

        loop {
            if let Some(error) = WEATHER.fetch_nws_alerts().await
                && error.detail.ends_with("out of bounds")
            {
                error!("Latitude and longitude are out of bounds for NWS alerts! Alerts will be now be disabled.");
                break;
            }
            
            tokio::time::sleep(std::time::Duration::from_secs(clamped_interval)).await;
        }
    });
}
