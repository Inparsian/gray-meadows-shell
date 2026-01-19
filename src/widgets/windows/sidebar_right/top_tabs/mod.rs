pub mod notifications;
pub mod audio;

use crate::widgets::common::tabs::{Tabs, TabsStack, TabSize};

pub fn new() -> (Tabs, TabsStack) {
    let tabs = Tabs::new(TabSize::Normal, true);
    let tabs_stack = TabsStack::new(&tabs, Some("sidebar-right-top-tabs-content"));

    tabs.add_tab(
        "notifications",
        "notifications".to_owned(),
        Some("notifications"),
    );

    tabs_stack.add_tab(
        Some("notifications"),
        &notifications::new(),
    );

    tabs.add_tab(
        "audio",
        "audio".to_owned(),
        Some("volume_up"),
    );

    tabs_stack.add_tab(
        Some("audio"),
        &audio::new(),
    );
    
    tabs.current_tab.set(Some("audio".to_owned()));

    (tabs, tabs_stack)
}