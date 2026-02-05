mod views {
    pub mod overview;
}

use gtk::prelude::*;

use crate::widgets::common::tabs::{Tabs, TabSize};
use self::views::overview::overview;

#[derive(Clone)]
pub struct CompactStatRow {
    pub container: gtk::Box,
    pub value: gtk::Label,
    pub secondary_value: gtk::Label, // e.g. cpu temp next to cpu usage
}

impl CompactStatRow {
    pub fn new(label_text: &str, with_secondary: bool) -> Self {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        container.set_css_classes(&["bar-sysstats-compact-stat-row"]);
        let label = gtk::Label::new(None);
        label.set_text(label_text);
        label.set_hexpand(true);
        label.set_halign(gtk::Align::Start);
        label.set_css_classes(&["bar-sysstats-compact-stat-row-label"]);
        let value = gtk::Label::new(None);
        value.set_css_classes(&["bar-sysstats-compact-stat-row-value"]);
        value.set_xalign(1.0);
        let secondary_value = gtk::Label::new(None);
        secondary_value.set_css_classes(&["bar-sysstats-compact-stat-row-secondary-value"]);
        secondary_value.set_xalign(1.0);

        container.append(&label);
        container.append(&value);

        if with_secondary {
            container.append(&secondary_value);
        }

        Self {
            container,
            value,
            secondary_value,
        }
    }

    pub fn set_value(&self, text: &str) {
        self.value.set_text(text);
    }

    pub fn set_secondary_value(&self, text: &str) {
        self.secondary_value.set_text(format!("({})", text).as_str());
    }
}

pub fn extended() -> gtk::Box {
    let tabs = Tabs::new(TabSize::Normal, true, Some("bar-sysstats-extended-tabs-stack"));
    tabs.set_current_tab(Some("overview"));
    tabs.add_tab("overview", "overview", Some("overview"), &overview());
    tabs.group()
        .spacing(8)
        .class_name("bar-sysstats-extended")
        .build()
}