use gtk4::prelude::*;

use crate::services::weather::schemas::openmeteo::OpenMeteoResponse;

pub struct WeatherToday {
    pub wind_speed: gtk4::Label,
    pub wind_direction: gtk4::Label,
    pub wind_gusts: gtk4::Label,
    pub pressure_msl: gtk4::Label,
    pub surface_pressure: gtk4::Label,
    pub relative_humidity: gtk4::Label,
    pub cloud_cover: gtk4::Label,
    pub precipitation_sum: gtk4::Label,
    pub rain_sum: gtk4::Label,
    pub showers_sum: gtk4::Label,
    pub snowfall_sum: gtk4::Label,
}

impl Default for WeatherToday {
    fn default() -> Self {
        let wind_speed = gtk4::Label::new(None);
        wind_speed.set_css_classes(&["weather-today-value"]);
        let wind_direction = gtk4::Label::new(None);
        wind_direction.set_css_classes(&["weather-today-secondary-value"]);
        let wind_gusts = gtk4::Label::new(None);
        wind_gusts.set_css_classes(&["weather-today-wind-gust"]);
        wind_gusts.set_xalign(0.0);
        let pressure_msl = gtk4::Label::new(None);
        pressure_msl.set_css_classes(&["weather-today-value"]);
        pressure_msl.set_xalign(0.0);
        let surface_pressure = gtk4::Label::new(None);
        surface_pressure.set_css_classes(&["weather-today-secondary-value"]);
        surface_pressure.set_xalign(0.0);
        let relative_humidity = gtk4::Label::new(None);
        relative_humidity.set_css_classes(&["weather-today-value"]);
        relative_humidity.set_xalign(0.0);
        let cloud_cover = gtk4::Label::new(None);
        cloud_cover.set_css_classes(&["weather-today-value"]);
        cloud_cover.set_xalign(0.0);
        let precipitation_sum = gtk4::Label::new(None);
        precipitation_sum.set_css_classes(&["weather-today-value"]);
        precipitation_sum.set_xalign(0.0);
        let rain_sum = gtk4::Label::new(None);
        let showers_sum = gtk4::Label::new(None);
        let snowfall_sum = gtk4::Label::new(None);

        Self {
            wind_speed,
            wind_direction,
            wind_gusts,
            pressure_msl,
            surface_pressure,
            relative_humidity,
            cloud_cover,
            precipitation_sum,
            rain_sum,
            showers_sum,
            snowfall_sum,
        }
    }
}

impl WeatherToday {
    pub fn update(&self, forecast: &OpenMeteoResponse) {
        self.wind_speed.set_label(&format!("{:.1} {}", forecast.current.wind_speed_10m, forecast.current_units.wind_speed_10m));
        self.wind_direction.set_label(&format!("({}°)", forecast.current.wind_direction_10m));
        self.wind_gusts.set_label(&format!("{:.1} {}", forecast.current.wind_gusts_10m, forecast.current_units.wind_gusts_10m));
        self.pressure_msl.set_label(&format!("{:.1} {}", forecast.current.pressure_msl, forecast.current_units.pressure_msl));
        self.surface_pressure.set_label(&format!("{:.1} {} surface", forecast.current.surface_pressure, forecast.current_units.surface_pressure));
        self.relative_humidity.set_label(&format!("{}{}", forecast.current.relative_humidity_2m, forecast.current_units.relative_humidity_2m));
        self.cloud_cover.set_label(&format!("{}{}", forecast.current.cloud_cover, forecast.current_units.cloud_cover));
        self.precipitation_sum.set_label(&format!("{:.1} {}", forecast.current.precipitation, forecast.current_units.precipitation));
        self.rain_sum.set_label(&format!("{:.1} {}", forecast.current.rain, forecast.current_units.rain));
        self.showers_sum.set_label(&format!("{:.1} {}", forecast.current.showers, forecast.current_units.showers));
        self.snowfall_sum.set_label(&format!("{:.1} {}", forecast.current.snowfall, forecast.current_units.snowfall));
    }

    pub fn build(&self) -> gtk4::Box {
        view! {
            root = gtk4::Box {
                set_css_classes: &["weather-today"],
                set_orientation: gtk4::Orientation::Vertical,
                set_hexpand: true,
                set_spacing: 4,

                gtk4::Box {
                    set_css_classes: &["weather-today-row"],
                    set_homogeneous: true,
                    set_hexpand: true,
                    set_spacing: 4,

                    gtk4::Box {
                        set_hexpand: true,
                        set_spacing: 8,

                        gtk4::Label {
                            set_css_classes: &["weather-today-icon"],
                            set_label: "air",
                        },

                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Vertical,
                            set_hexpand: true,
                            set_spacing: 2,

                            gtk4::Box {
                                set_spacing: 4,

                                append: &self.wind_speed,
                                append: &self.wind_direction,
                            },

                            append: &self.wind_gusts,
                        },
                    },

                    gtk4::Box {
                        set_hexpand: true,
                        set_spacing: 8,

                        gtk4::Label {
                            set_css_classes: &["weather-today-icon"],
                            set_label: "speed",
                        },

                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Vertical,
                            set_hexpand: true,
                            set_spacing: 2,

                            append: &self.pressure_msl,
                            append: &self.surface_pressure,
                        },
                    },
                },

                gtk4::Box {
                    set_css_classes: &["weather-today-row"],
                    set_homogeneous: true,
                    set_hexpand: true,
                    set_spacing: 4,

                    gtk4::Box {
                        set_hexpand: true,
                        set_spacing: 8,

                        gtk4::Label {
                            set_css_classes: &["weather-today-icon"],
                            set_label: "humidity_percentage",
                        },

                        append: &self.relative_humidity,
                    },

                    gtk4::Box {
                        set_hexpand: true,
                        set_spacing: 8,

                        gtk4::Label {
                            set_css_classes: &["weather-today-icon"],
                            set_label: "cloud",
                        },

                        append: &self.cloud_cover,
                    },
                },

                gtk4::Box {
                    set_css_classes: &["weather-today-row"],
                    set_homogeneous: true,
                    set_hexpand: true,
                    set_spacing: 4,

                    gtk4::Box {
                        set_hexpand: true,
                        set_spacing: 8,

                        gtk4::Label {
                            set_css_classes: &["weather-today-icon"],
                            set_label: "umbrella",
                        },

                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Vertical,
                            set_hexpand: true,
                            set_spacing: 2,

                            append: &self.precipitation_sum,

                            gtk4::Box {
                                set_spacing: 4,

                                gtk4::Box {
                                    set_css_classes: &["weather-today-secondary-value"],
                                    set_spacing: 2,

                                    gtk4::Label {
                                        set_css_classes: &["material-icons"],
                                        set_label: "water_drop",
                                    },
                                    append: &self.rain_sum,
                                },

                                gtk4::Box {
                                    set_css_classes: &["weather-today-secondary-value"],
                                    set_spacing: 2,

                                    gtk4::Label {
                                        set_css_classes: &["material-icons"],
                                        set_label: "shower",
                                    },
                                    append: &self.showers_sum,
                                },

                                gtk4::Box {
                                    set_css_classes: &["weather-today-secondary-value"],
                                    set_spacing: 2,

                                    gtk4::Label {
                                        set_css_classes: &["material-icons"],
                                        set_label: "ac_unit",
                                    },
                                    append: &self.snowfall_sum,
                                },
                            },
                        },
                    },
                },
            }
        }

        root
    }
}