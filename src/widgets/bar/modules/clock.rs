use futures_signals::signal::SignalExt;
use gtk4::prelude::*;

use crate::{
    singletons::date_time::DATE_TIME,
    widgets::{self, bar::wrapper::BarModuleWrapper}
};

pub fn new() -> gtk4::Box {
    view! {
        time_label = gtk4::Label {},
        date_label = gtk4::Label {},
        widget = gtk4::Box {
            set_css_classes: &["bar-widget"],
            set_hexpand: false,

            append: &time_label,
            widgets::dot_separator::new() {},
            append: &date_label
        }
    }

    let date_time_future = DATE_TIME.signal_cloned().for_each(move |date_time| {
        time_label.set_label(&date_time.time);
        date_label.set_label(&date_time.date);

        async {}
    });

    gtk4::glib::spawn_future_local(date_time_future);

    BarModuleWrapper::new(&widget).get_widget()
}