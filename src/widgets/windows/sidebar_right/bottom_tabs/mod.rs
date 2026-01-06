pub mod calendar;

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

    tabs.current_tab.set(Some("calendar".to_owned()));

    (tabs, tabs_stack)
}