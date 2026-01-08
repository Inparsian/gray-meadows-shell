pub mod schemas {
    pub mod openmeteo;
}

use std::sync::LazyLock;
use futures_signals::signal::Mutable;
use reqwest::Client;

use crate::config::read_config;
use crate::sql::wrappers::weather::{get_weather_forecast, set_weather_forecast};
use self::schemas::openmeteo::OpenMeteoResponse;

pub static WEATHER: LazyLock<Weather> = LazyLock::new(Weather::default);

pub const WMO_CODES: &[WmoCode] = &[
    WmoCode { code: 0, text: "Clear sky", short_text: "clear", day_icon: "clear_day", night_icon: "moon_stars" },
    WmoCode { code: 1, text: "Mainly clear", short_text: "clearish", day_icon: "partly_cloudy_day", night_icon: "partly_cloudy_night" },
    WmoCode { code: 2, text: "Partly cloudy", short_text: "cloudyish", day_icon: "partly_cloudy_day", night_icon: "partly_cloudy_night" },
    WmoCode { code: 3, text: "Mostly cloudy", short_text: "cloudy", day_icon: "cloud", night_icon: "cloud" },
    WmoCode { code: 45, text: "Fog", short_text: "fog", day_icon: "foggy", night_icon: "foggy" },
    WmoCode { code: 48, text: "Fog", short_text: "fog", day_icon: "foggy", night_icon: "foggy" },
    WmoCode { code: 51, text: "Light drizzle", short_text: "drizzle (l)", day_icon: "rainy_light", night_icon: "rainy_light" },
    WmoCode { code: 53, text: "Drizzle", short_text: "drizzle", day_icon: "rainy", night_icon: "rainy" },
    WmoCode { code: 55, text: "Dense drizzle", short_text: "drizzle (h)", day_icon: "rainy_heavy", night_icon: "rainy_heavy" },
    WmoCode { code: 56, text: "Light freezing drizzle", short_text: "drizzle (fl)", day_icon: "rainy_light", night_icon: "rainy_light" },
    WmoCode { code: 57, text: "Dense freezing drizzle", short_text: "drizzle (fh)", day_icon: "rainy_heavy", night_icon: "rainy_heavy" },
    WmoCode { code: 61, text: "Slight rain", short_text: "rain (l)", day_icon: "rainy_light", night_icon: "rainy_light" },
    WmoCode { code: 63, text: "Rain", short_text: "rain", day_icon: "rainy", night_icon: "rainy" },
    WmoCode { code: 65, text: "Heavy rain", short_text: "rain (h)", day_icon: "rainy_heavy", night_icon: "rainy_heavy" },
    WmoCode { code: 66, text: "Light freezing rain", short_text: "rain (fl)", day_icon: "rainy_light", night_icon: "rainy_light" },
    WmoCode { code: 67, text: "Heavy freezing rain", short_text: "rain (fh)", day_icon: "rainy_heavy", night_icon: "rainy_heavy" },
    WmoCode { code: 71, text: "Slight snow", short_text: "snow (l)", day_icon: "weather_snowy", night_icon: "weather_snowy" },
    WmoCode { code: 73, text: "Snow", short_text: "snow", day_icon: "weather_snowy", night_icon: "weather_snowy" },
    WmoCode { code: 75, text: "Heavy snow", short_text: "snow (h)", day_icon: "weather_snowy", night_icon: "weather_snowy" },
    WmoCode { code: 77, text: "Snow grains", short_text: "snow grains", day_icon: "weather_snowy", night_icon: "weather_snowy" },
    WmoCode { code: 80, text: "Light rain showers", short_text: "showers (l)", day_icon: "rainy_light", night_icon: "rainy_light" },
    WmoCode { code: 81, text: "Rain showers", short_text: "showers", day_icon: "rainy", night_icon: "rainy" },
    WmoCode { code: 82, text: "Heavy rain showers", short_text: "showers (h)", day_icon: "rainy_heavy", night_icon: "rainy_heavy" },
    WmoCode { code: 85, text: "Light snow showers", short_text: "snow showers (l)", day_icon: "weather_snowy", night_icon: "weather_snowy" },
    WmoCode { code: 86, text: "Heavy snow showers", short_text: "snow showers (h)", day_icon: "weather_snowy", night_icon: "weather_snowy" },
    WmoCode { code: 95, text: "Thunderstorm", short_text: "storm", day_icon: "thunderstorm", night_icon: "thunderstorm" },
    WmoCode { code: 96, text: "Thunderstorm, slight hail", short_text: "storm (lH)", day_icon: "thunderstorm", night_icon: "thunderstorm" },
    WmoCode { code: 99, text: "Thunderstorm, heavy hail", short_text: "storm (hH)", day_icon: "thunderstorm", night_icon: "thunderstorm" },
];

const OPENMETEO_API_URL: &str = "https://api.open-meteo.com/v1/forecast";

#[allow(dead_code)]
pub struct WmoCode {
    pub code: i64,
    pub text: &'static str,
    pub short_text: &'static str,
    pub day_icon: &'static str,
    pub night_icon: &'static str,
}

#[derive(Clone, Debug)]
pub struct Weather {
    pub client: Client,
    pub last_response: Mutable<Option<OpenMeteoResponse>>,
}

impl Default for Weather {
    fn default() -> Self {
        Self {
            client: Client::builder()
                .user_agent("GrayMeadowsShell/1.0")
                .build()
                .unwrap_or_default(),
            last_response: Mutable::new(None),
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
            ("temperature_unit", weather_config.temperature_unit.clone()),
            ("wind_speed_unit", weather_config.speed_unit.clone()),
            ("precipitation_unit", weather_config.precipitation_unit.clone()),
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
                    let _ = set_weather_forecast(&weather_data);
                    self.last_response.set(Some(weather_data));
                }

                Err(err) => {
                    eprintln!("Failed to parse weather data: {:#?}", err);
                }
            },

            Err(err) => {
                eprintln!("Failed to fetch weather data: {:#?}", err);
            }
        }
    }

    /// Returns the last cached forecast's age in seconds, if there was a hit.
    pub fn cache_check(&self) -> Option<i64> {
        if let Ok(Some((fetched_at, forecast))) = get_weather_forecast() {
            let now = chrono::Utc::now().naive_utc();
            let elapsed = now.signed_duration_since(fetched_at).num_seconds();
            println!("[weather] Got a cache hit! Forecast fetched {} seconds ago.", elapsed);
            self.last_response.set(Some(forecast));
            Some(elapsed)
        } else {
            None
        }
    }
}

#[allow(dead_code)]
pub fn get_wmo_code(code: i64) -> Option<&'static WmoCode> {
    WMO_CODES.iter().find(|wmo_code| wmo_code.code == code)
}

pub fn activate() {
    tokio::spawn(async move {
        let weather_config = {
            let config = read_config();
            config.weather.clone()
        };

        if !weather_config.enabled {
            return;
        }

        let clamped_interval = weather_config.refresh_interval.max(600);
        if let Some(elapsed) = WEATHER.cache_check() && elapsed < clamped_interval as i64 {
            let sleep_duration = clamped_interval as i64 - elapsed;
            println!("[weather] Sleeping for {sleep_duration} seconds...");
            tokio::time::sleep(std::time::Duration::from_secs(sleep_duration as u64)).await;
        }

        loop {
            WEATHER.fetch().await;
            tokio::time::sleep(std::time::Duration::from_secs(clamped_interval)).await;
        }
    });
}