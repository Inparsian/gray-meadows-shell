use futures_signals::signal::SignalExt as _;
use gtk4::prelude::*;

use crate::singletons::date_time::DATE_TIME;
use crate::widgets::common::dot_separator;
use super::super::wrapper::SimpleBarModuleWrapper;

pub fn new() -> gtk4::Box {
    view! {
        time_label = gtk4::Label {},
        date_label = gtk4::Label {},
        widget = gtk4::Box {
            set_css_classes: &["bar-widget"],
            set_hexpand: false,

            append: &time_label,
            append: &dot_separator::new(),
            append: &date_label
        }
    }

    gtk4::glib::spawn_future_local(signal_cloned!(DATE_TIME, (date_time) {
        time_label.set_label(&date_time.time);
        date_label.set_label(&date_time.date);
    }));

    SimpleBarModuleWrapper::new(&widget).get_widget()
}