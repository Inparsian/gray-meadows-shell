use gtk4::prelude::*;
use relm4::RelmIterChildrenExt as _;

use crate::sql::wrappers::commands;

#[derive(Debug, Clone)]
pub struct OverviewFrequentWindow {
    pub widget: gtk4::Box,
    pub children: gtk4::Box,
}

impl OverviewFrequentWindow {
    pub fn new() -> Self {
        let (widget, children) = super::build_window("Frequently Launched");

        Self {
            widget,
            children
        }
    }

    pub async fn update(&self) {
        self.children.iter_children().for_each(|child| {
            self.children.remove(&child);
        });

        // Get the frequently launched applications
        if let Ok(entries) = commands::get_top_commands(10).await {
            for entry in entries {
                if let Some(button) = super::make_item_from_command(&entry.0) {
                    self.children.append(&button);
                }
            }
        }
    }
}