pub mod popup;
pub mod overview;
pub mod session;
pub mod sidebar_left;
pub mod sidebar_right;

use gtk4::prelude::*;
use gtk4_layer_shell::LayerShell;

use crate::{APP_LOCAL, ipc, singletons::hyprland, widgets::windows::popup::Popup};

fn with(window: &str, callback: impl FnOnce(&gtk4::ApplicationWindow)) {
    APP_LOCAL.with(|app| {
        let borrow_attempt = match window {
            "overview" => app.borrow().overview_window.borrow().as_ref().cloned(),
            "session" => app.borrow().session_window.borrow().as_ref().cloned(),
            _ => None,
        };

        if let Some(win) = borrow_attempt {
            callback(&win);
        }
    });
}

fn with_popup(window: &str, callback: impl FnOnce(&Popup)) {
    APP_LOCAL.with(|app| {
        let borrow_attempt = app.borrow().popup_windows.borrow().get(window).cloned();

        if let Some(popup) = borrow_attempt {
            callback(&popup);
        }
    });
}

fn popup_exists(window: &str) -> bool {
    let mut exists = false;
    APP_LOCAL.with(|app| {
        exists = app.borrow().popup_windows.borrow().contains_key(window);
    });
    exists
}

pub fn show(window: &str) -> bool {
    if popup_exists(window) {
        with_popup(window, |popup| popup.show());
        return true;
    }
    
    with(window, |win| {
        let monitor = hyprland::get_active_monitor();
        win.set_monitor(monitor.as_ref());
        win.show();
    });
    true
}

pub fn hide(window: &str) -> bool {
    if popup_exists(window) {
        with_popup(window, |popup| popup.hide_without_checking_options());
        return false;
    }

    with(window, |win| win.hide());
    false
}

pub fn toggle(window: &str) -> bool {
    let mut was_visible = false;
    if popup_exists(window) {
        with_popup(window, |popup| {
            was_visible = if popup.is_visible() {
                popup.hide_without_checking_options();
                true
            } else {
                popup.show();
                false
            }
        });
        return !was_visible;
    }

    with(window, |win| {
        was_visible = if win.is_visible() {
            win.hide();
            true
        } else {
            let monitor = hyprland::get_active_monitor();
            win.set_monitor(monitor.as_ref());
            win.show();
            false
        }
    });
    !was_visible
}

pub fn hide_all_popups() {
    APP_LOCAL.with(|app| {
        for popup in app.borrow().popup_windows.borrow().values() {
            if popup.is_visible() {
                popup.hide_without_checking_options();
            }
        }
    });
}

pub fn listen_for_ipc_messages() {
    ipc::listen_for_messages_local(|message| {
        if let Some(window_name) = message.strip_prefix("show_") {
            show(window_name);
        } else if let Some(window_name) = message.strip_prefix("hide_") {
            hide(window_name);
        } else if let Some(window_name) = message.strip_prefix("toggle_") {
            let toggled = toggle(window_name);

            if window_name == "overview" && toggled {
                let _ = ipc::client::send_message("update_overview_windows");
            }
        }
    });
}