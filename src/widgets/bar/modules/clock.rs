use futures_signals::signal::SignalExt as _;
use gtk::prelude::*;

use crate::services::date_time::DATE_TIME;
use crate::widgets::common::dot_separator;
use super::super::base::BarModule;

pub fn new() -> BarModule {
    view! {
        time_label = gtk::Label {},
        date_label = gtk::Label {},
        widget = gtk::Box {
            set_hexpand: false,
            append: &time_label,
            append: &dot_separator::new(),
            append: &date_label
        }
    }

    glib::spawn_future_local(signal_cloned!(DATE_TIME, (date_time) {
        time_label.set_label(&date_time.time);
        date_label.set_label(&date_time.date);
    }));

    BarModule::builder()
        .minimal_widget(&widget)
        .build()
}