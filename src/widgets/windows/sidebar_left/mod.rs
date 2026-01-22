pub mod modules;

use gtk4::prelude::*;

use crate::ipc;
use crate::widgets::common::tabs::{TabSize, Tabs};
use super::popup::{PopupWindow, PopupMargin, PopupOptions};

pub fn new(application: &libadwaita::Application) -> PopupWindow {
    let tabs = Tabs::new(TabSize::Large, true, None);
    tabs.set_current_tab(Some("ai"));
    tabs.add_tab("translate", "translate", Some("g_translate"), &modules::translate::new());
    tabs.add_tab("color picker", "color_picker", Some("palette"), &modules::color_picker::new());
    tabs.add_tab("ai", "ai", Some("chat"), &modules::ai::new());

    view! {
        left_sidebar_expand_button_label = gtk4::Label {
            set_css_classes: &["left-sidebar-expand-button-icon"],
            set_label: "expand_content",
        },

        left_sidebar_expand_button = gtk4::Button {
            set_css_classes: &["left-sidebar-expand-button"],
            set_halign: gtk4::Align::End,
            set_hexpand: true,
            set_child: Some(&left_sidebar_expand_button_label),
        },

        left_sidebar_box = gtk4::Box {
            set_css_classes: &["left-sidebar-box"],
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 12,
            set_hexpand: true,
            set_vexpand: true,

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 0,
                append: &tabs.select,
                append: &left_sidebar_expand_button,
            },
            append: &tabs.stack,
        },
    };

    let toggle_expand = {
        let left_sidebar_box = left_sidebar_box.clone();
        move || if left_sidebar_box.has_css_class("expanded") {
            left_sidebar_box.remove_css_class("expanded");
            left_sidebar_expand_button_label.set_label("expand_content");
        } else {
            left_sidebar_box.add_css_class("expanded");
            left_sidebar_expand_button_label.set_label("collapse_content");
        }
    };

    left_sidebar_expand_button.connect_clicked({
        let toggle_expand = toggle_expand.clone();
        move |_| toggle_expand()
    });

    ipc::listen_for_messages_local(move |message| {
        let mut split_whitespace_iterator = message.split_whitespace();
        if let Some(message) = split_whitespace_iterator.next() {
            match message {
                "change_left_sidebar_tab" => if let Some(tab) = split_whitespace_iterator.next()
                    && tabs.items.try_borrow().is_ok_and(|vec| vec.iter().any(|t| t.name == tab))
                {
                    tabs.set_current_tab(Some(tab));
                },

                "toggle_left_sidebar_expanded" => toggle_expand(),
                
                _ => {},
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