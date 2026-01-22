pub mod notifications;
pub mod audio;

use crate::widgets::common::tabs::{Tabs, TabSize};

pub fn new() -> Tabs {
    let tabs = Tabs::new(TabSize::Normal, true, Some("sidebar-right-top-tabs-content"));
    tabs.add_tab("notifications", "notifications", Some("notifications"), &notifications::new());
    tabs.add_tab("audio", "audio", Some("volume_up"), &audio::new());
    tabs.current_tab.set(Some("audio".to_owned()));
    tabs
}