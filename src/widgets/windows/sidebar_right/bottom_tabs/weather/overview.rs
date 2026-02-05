use gtk::prelude::*;

use crate::services::weather::get_wmo_code;
use crate::services::weather::schemas::openmeteo::OpenMeteoResponse;

pub struct WeatherOverview {
    pub current_icon: gtk::Label,
    pub actual_temp_label: gtk::Label,
    pub feels_like_label: gtk::Label,
    pub condition_label: gtk::Label,
    pub daily_high_label: gtk::Label,
    pub daily_low_label: gtk::Label,
}

impl Default for WeatherOverview {
    fn default() -> Self {
        let current_icon = gtk::Label::new(None);
        current_icon.set_css_classes(&["current-weather-icon"]);
        let actual_temp_label = gtk::Label::new(None);
        actual_temp_label.set_css_classes(&["current-weather-actual-temp"]);
        let feels_like_label = gtk::Label::new(None);
        feels_like_label.set_css_classes(&["current-weather-feels-like-temp"]);
        let condition_label = gtk::Label::new(None);
        condition_label.set_css_classes(&["current-weather-condition"]);
        condition_label.set_xalign(0.0);
        let daily_high_label = gtk::Label::new(None);
        let daily_low_label = gtk::Label::new(None);

        Self {
            current_icon,
            actual_temp_label,
            feels_like_label,
            condition_label,
            daily_high_label,
            daily_low_label,
        }
    }
}

impl WeatherOverview {
    pub fn update(&self, forecast: &OpenMeteoResponse) {
        let Some(wmo) = get_wmo_code(forecast.current.weather_code) else {
            return;
        };
        
        self.current_icon.set_label(wmo.get_icon(forecast.current.is_day == 1));
        self.actual_temp_label.set_label(&format!("{:.1}{}", forecast.current.temperature_2m, forecast.current_units.temperature_2m));
        self.feels_like_label.set_label(&format!("{:.1}{}", forecast.current.apparent_temperature, forecast.current_units.temperature_2m));
        self.condition_label.set_label(wmo.text);
        self.daily_high_label.set_label(&format!("{:.1}{}", forecast.daily.temperature_2m_max[0], forecast.daily_units.temperature_2m_max));
        self.daily_low_label.set_label(&format!("{:.1}{}", forecast.daily.temperature_2m_min[0], forecast.daily_units.temperature_2m_min));
    }

    pub fn build(&self) -> gtk::Box {
        view! {
            root = gtk::Box {
                set_css_classes: &["current-weather"],
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,
                set_spacing: 4,

                gtk::Box {
                    set_css_classes: &["current-weather-status"],
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,
                    set_spacing: 6,

                    append: &self.current_icon,
                    gtk::Box {
                        set_css_classes: &["current-weather-outlook"],
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 2,
                        set_valign: gtk::Align::Center,

                        gtk::Box {
                            set_css_classes: &["current-weather-temp"],
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 8,

                            append: &self.actual_temp_label,
                            append: &self.feels_like_label,
                        },
                        append: &self.condition_label,
                    },
                },

                gtk::Box {
                    set_css_classes: &["current-weather-other-temps"],
                    set_orientation: gtk::Orientation::Vertical,
                    set_valign: gtk::Align::Center,

                    gtk::Box {
                        set_css_classes: &["current-weather-high-temp"],
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 2,

                        gtk::Label {
                            set_css_classes: &["material-icons"],
                            set_label: "arrow_upward",
                        },
                        append: &self.daily_high_label,
                    },

                    gtk::Box {
                        set_css_classes: &["current-weather-low-temp"],
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 2,

                        gtk::Label {
                            set_css_classes: &["material-icons"],
                            set_label: "arrow_downward",
                        },
                        append: &self.daily_low_label,
                    },
                },
            }
        }

        root
    }
}