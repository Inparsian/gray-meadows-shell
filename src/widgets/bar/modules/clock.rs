use gtk4::prelude::*;

use crate::{reactivity, singletons};

pub fn new() -> gtk4::Box {
    relm4_macros::view! {
        widget = gtk4::Box {
            set_css_classes: &["bar-widget"],
            set_hexpand: false,

            reactivity::reactive_label(singletons::date_time::DATE_TIME.time.clone()) {
                set_hexpand: true,
                set_xalign: 0.5
            }
        }
    }

    widget
}