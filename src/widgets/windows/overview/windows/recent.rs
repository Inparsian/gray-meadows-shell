use gtk4::prelude::*;
use relm4::RelmIterChildrenExt as _;

use crate::sql::wrappers::commands;

pub struct OverviewRecentWindow {
    pub widget: gtk4::Box,
    pub children: gtk4::Box,
}

impl OverviewRecentWindow {
    pub fn new() -> Self {
        let (widget, children) = super::build_window("Recently Launched");

        Self {
            widget,
            children
        }
    }

    pub fn update(&self) {
        self.children.iter_children().for_each(|child| {
            self.children.remove(&child);
        });

        // Get the most recently launched applications
        if let Ok(entries) = commands::get_recent_commands(10) {
            for entry in entries {
                if let Some(button) = super::make_item_from_command(&entry.0) {
                    self.children.append(&button);
                }
            }
        }
    }
}