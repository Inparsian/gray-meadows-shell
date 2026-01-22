pub mod calendar;
pub mod weather;

use crate::widgets::common::tabs::{Tabs, TabSize};

pub fn new() -> Tabs {
    let tabs = Tabs::new(TabSize::Tiny, true, Some("sidebar-right-bottom-tabs-content"));
    tabs.add_tab("calendar", "calendar", Some("calendar_today"), &calendar::new());
    tabs.add_tab("weather", "weather", Some("partly_cloudy_day"), &weather::new());
    tabs.current_tab.set(Some("weather".to_owned()));
    tabs
}