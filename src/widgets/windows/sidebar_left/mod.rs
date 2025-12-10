pub mod modules;

use gtk4::prelude::*;

use crate::ipc;
use crate::widgets::common::tabs::{TabSize, Tabs, TabsStack};
use super::popup::{PopupWindow, PopupMargin, PopupOptions};

pub fn new(application: &libadwaita::Application) -> PopupWindow {
    let tabs = Tabs::new(TabSize::Large, true);
    tabs.current_tab.set(Some("color_picker".to_owned()));
    tabs.add_tab("translate", "translate".to_owned(), Some("g_translate"));
    tabs.add_tab("color picker", "color_picker".to_owned(), Some("palette"));

    let tabs_stack = TabsStack::new(&tabs, None);
    tabs_stack.add_tab(Some("translate"), &modules::translate::new());
    tabs_stack.add_tab(Some("color_picker"), &modules::color_picker::new());

    view! {
        left_sidebar_box = gtk4::Box {
            set_css_classes: &["left-sidebar-box"],
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 12,
            set_hexpand: true,
            set_vexpand: true,

            append: &tabs.widget,
            append: &tabs_stack.widget
        },
    };

    ipc::listen_for_messages_local(move |message| {
        let mut split_whitespace_iterator = message.split_whitespace();
        if let Some(message) = split_whitespace_iterator.next() {
            if message == "change_left_sidebar_tab" {
                if let Some(tab) = split_whitespace_iterator.next() {
                    if tabs.items.try_borrow().is_ok_and(|vec| vec.iter().any(|t| t.name == tab)) {
                        tabs.current_tab.set(Some(tab.to_owned()));
                    }
                }
            }
        }
    });

    PopupWindow::new(
        application,
        &["left-sidebar-window"],
        &left_sidebar_box,
        PopupOptions {
            anchor_left: true,
            anchor_right: false,
            anchor_top: true,
            anchor_bottom: true,
        },
        300,
        400,
        PopupMargin {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
        },
        gtk4::RevealerTransitionType::SlideRight,
        200,
    )
}