use gtk4::prelude::*;

use crate::{widgets, reactivity, singletons};

pub fn new() -> gtk4::Box {
    relm4_macros::view! {
        widget = gtk4::Box {
            set_css_classes: &["bar-widget"],
            set_hexpand: false,

            reactivity::reactive_label(singletons::date_time::DATE_TIME.time.clone()) {},
            widgets::dot_separator::new() {},
            reactivity::reactive_label(singletons::date_time::DATE_TIME.date.clone()) {}
        }
    }

    widget
}