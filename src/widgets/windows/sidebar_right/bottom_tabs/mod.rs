pub mod calendar;
pub mod weather;

use crate::widgets::common::tabs::{Tabs, TabsStack, TabSize};

pub fn new() -> (Tabs, TabsStack) {
    let tabs = Tabs::new(TabSize::Tiny, true);
    let tabs_stack = TabsStack::new(&tabs, Some("sidebar-right-bottom-tabs-content"));

    tabs.add_tab(
        "calendar",
        "calendar".to_owned(),
        Some("calendar_today"),
    );

    tabs_stack.add_tab(
        Some("calendar"),
        &calendar::new(),
    );

    tabs.add_tab(
        "weather",
        "weather".to_owned(),
        Some("partly_cloudy_day"),
    );

    tabs_stack.add_tab(
        Some("weather"),
        &weather::new(),
    );

    tabs.current_tab.set(Some("weather".to_owned()));

    (tabs, tabs_stack)
}