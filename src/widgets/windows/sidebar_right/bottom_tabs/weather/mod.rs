mod overview;
mod today;
mod week;
mod alerts;

use gtk::prelude::*;
use futures_signals::signal::SignalExt as _;

use crate::services::weather::WEATHER;
use crate::widgets::common::tabs::{Tabs, TabSize};

pub fn new() -> gtk::Box {
    let overview = overview::WeatherOverview::default();
    let today = today::WeatherToday::default();
    let week = week::WeatherWeek::default();
    let alerts = alerts::WeatherAlerts::default();

    let tabs = Tabs::new(TabSize::Tiny, false, Some("weather-tab-tabs"));
    tabs.add_tab("today", "today", None, &today.build());
    tabs.add_tab("week", "week", None, &week.bx);
    tabs.add_tab("alerts", "alerts", None, &alerts.root);
    tabs.current_tab.set(Some("today".to_owned()));

    view! {
        root = gtk::Box {
            set_css_classes: &["weather-tab-root"],
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 4,

            append: &overview.build(),
            append: &tabs.group()
                .spacing(4)
                .build(),
        }
    }

    glib::spawn_future_local(signal_cloned!(WEATHER.last_response, (forecast) {
        if let Some(forecast) = &forecast {
            overview.update(forecast);
            today.update(forecast);
            week.update(forecast);
        }
    }));
    
    glib::spawn_future_local(signal_cloned!(WEATHER.last_alerts_response, (weather_alerts) {
        if let Some(weather_alerts) = &weather_alerts {
            alerts.update(weather_alerts);
        }
    }));

    root
}