use gtk4::prelude::*;
use relm4::RelmIterChildrenExt;

use crate::SQL_CONNECTION;

pub struct OverviewFrequentWindow {
    pub widget: gtk4::Box,
    pub children: gtk4::Box,
}

impl OverviewFrequentWindow {
    pub fn new() -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        widget.set_css_classes(&["overview-window"]);
        widget.set_hexpand(true);
        widget.set_vexpand(true);
        widget.set_halign(gtk4::Align::Center);
        widget.set_valign(gtk4::Align::Center);

        let header = gtk4::Label::new(Some("Frequently Launched"));
        header.set_css_classes(&["overview-window-header"]);
        widget.append(&header);

        let children = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        children.set_css_classes(&["overview-window-children"]);
        children.set_hexpand(true);
        children.set_vexpand(true);
        widget.append(&children);

        Self {
            widget,
            children
        }
    }

    pub fn update(&self) {
        self.children.iter_children().for_each(|child| {
            self.children.remove(&child);
        });

        // Get the frequently launched applications
        if let Some(connection) = SQL_CONNECTION.get() {
            if let Ok(entries) = connection.get_top_commands(10) {
                for entry in entries {
                    if let Some(button) = super::make_item_from_command(&entry.0) {
                        self.children.append(&button);
                    }
                }
            }
        }
    }
}