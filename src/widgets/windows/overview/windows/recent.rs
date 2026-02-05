use gtk::prelude::*;
use relm4::RelmIterChildrenExt as _;

use crate::services::apps::runs;

#[derive(Debug, Clone)]
pub struct OverviewRecentWindow {
    pub widget: gtk::Box,
    pub children: gtk::Box,
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

        let mut children = 0;
        for entry in runs::get_most_recent_commands() {
            if children >= 10 {
                break;
            }
            
            if let Some(button) = super::make_item_from_command(&entry.command) {
                self.children.append(&button);
                children += 1;
            }
        }
    }
}