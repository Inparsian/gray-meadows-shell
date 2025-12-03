mod header;
mod quicktoggle;

use gtk4::prelude::*;

use crate::widgets::windows::popup::{PopupWindow, PopupMargin, PopupOptions};

pub fn new(application: &libadwaita::Application) -> PopupWindow {
    let header = header::new();

    view! {
        quick_toggles = gtk4::Box {
            set_css_classes: &["sidebar-right-quicktoggles"],
            set_spacing: 4,
            set_orientation: gtk4::Orientation::Horizontal,
            set_hexpand: true,
            set_vexpand: false,

            append: &quicktoggle::keybinds::new(),
            append: &quicktoggle::gamemode::new(),
        },

        right_sidebar_box = gtk4::Box {
            set_css_classes: &["right-sidebar-box"],
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 12,
            set_hexpand: true,
            set_vexpand: true,

            append: &header,
            append: &quick_toggles
        },
    };

    PopupWindow::new(
        application,
        &["right-sidebar-window"],
        &right_sidebar_box,
        PopupOptions {
            anchor_left: false,
            anchor_right: true,
            anchor_top: true,
            anchor_bottom: true,
        },
        400,
        100,
        PopupMargin {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
        },
        gtk4::RevealerTransitionType::SlideLeft,
        200,
    )
}