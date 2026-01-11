mod overview;

use gtk4::prelude::*;
use futures_signals::signal::SignalExt as _;

use crate::singletons::weather::WEATHER;

pub fn new() -> gtk4::Box {
    let overview = overview::WeatherOverview::default();

    view! {
        root = gtk4::Box {
            set_css_classes: &["weather-tab-root"],
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 4,

            append: &overview.build(),
        }
    }

    gtk4::glib::spawn_future_local(signal_cloned!(WEATHER.last_response, (forecast) {
        if let Some(forecast) = &forecast {
            overview.update(forecast);
        }
    }));

    root
}