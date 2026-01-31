use gtk4::prelude::*;
use relm4::RelmRemoveAllExt as _;

use crate::services::weather::{get_daily_at, get_wmo_code};
use crate::services::weather::schemas::openmeteo::OpenMeteoResponse;

pub struct WeatherWeek {
    pub bx: gtk4::Box,
}

impl Default for WeatherWeek {
    fn default() -> Self {
        let bx = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        bx.set_css_classes(&["weather-daily-forecast"]);

        Self {
            bx,
        }
    }
}

impl WeatherWeek {
    pub fn update(&self, forecast: &OpenMeteoResponse) {
        self.bx.remove_all();

        for item in forecast.daily.time.iter().enumerate().filter_map(|(i, _)| get_daily_at(forecast, i)) {
            let Some(wmo) = get_wmo_code(item.weather_code) else {
                continue;
            };
            
            let weekday = chrono::NaiveDate::parse_from_str(&item.time, "%Y-%m-%d")
                .map_or_else(|_| "nil".to_owned(), |d| d.format("%A").to_string())
                [..3].to_lowercase();

            view! {
                day_bx = gtk4::Box {
                    set_css_classes: &["weather-forecast-day"],
                    set_orientation: gtk4::Orientation::Horizontal,
                    set_spacing: 6,
                    set_hexpand: true,

                    gtk4::Label {
                        set_css_classes: &["weather-forecast-day-name"],
                        set_label: &weekday,
                    },

                    gtk4::Label {
                        set_css_classes: &["weather-forecast-day-icon"],
                        set_label: &wmo.day_icon,
                    },

                    gtk4::Label {
                        set_css_classes: &["weather-forecast-day-outlook"],
                        set_xalign: 0.0,
                        set_hexpand: true,
                        set_label: &wmo.short_text,
                    },

                    gtk4::Box {
                        set_css_classes: &["weather-forecast-high"],
                        set_spacing: 2,

                        gtk4::Label {
                            set_css_classes: &["material-icons"],
                            set_label: "arrow_upward",
                        },

                        gtk4::Label {
                            set_label: &format!("{:.1}{}", item.temperature_2m_max, forecast.daily_units.temperature_2m_max),
                        },
                    },

                    gtk4::Box {
                        set_css_classes: &["weather-forecast-low"],
                        set_spacing: 2,

                        gtk4::Label {
                            set_css_classes: &["material-icons"],
                            set_label: "arrow_downward",
                        },

                        gtk4::Label {
                            set_label: &format!("{:.1}{}", item.temperature_2m_min, forecast.daily_units.temperature_2m_min),
                        },
                    },
                }
            }

            self.bx.append(&day_bx);
        }
    }
}