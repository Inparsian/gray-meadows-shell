use gtk4::prelude::*;
use gtk4_layer_shell::LayerShell;

use crate::{APP_LOCAL, ipc, singletons::hyprland, widgets::popup::Popup};

#[derive(Clone, Eq, PartialEq)]
pub enum Window {
    Overview,
    Session
}

impl Window {
    fn with(&self, callback: impl FnOnce(&gtk4::ApplicationWindow)) {
        APP_LOCAL.with(|app| {
            let borrow_attempt = match self {
                Window::Overview => app.borrow().overview_window.borrow().as_ref().cloned(),
                Window::Session => app.borrow().session_window.borrow().as_ref().cloned()
            };

            if let Some(win) = borrow_attempt {
                callback(&win);
            }
        });
    }

    pub fn show(&self) -> bool {
        self.with(|win| {
            let monitor = hyprland::get_active_monitor();
            win.set_monitor(monitor.as_ref());
            win.show();
        });
        true
    }

    pub fn hide(&self) -> bool {
        self.with(|win| win.hide());
        false
    }

    pub fn toggle(&self) -> bool {
        let mut was_visible = false;
        self.with(|win| {
            if win.is_visible() {
                win.hide();
                was_visible = true;
            } else {
                let monitor = hyprland::get_active_monitor();
                win.set_monitor(monitor.as_ref());
                win.show();
                was_visible = false;
            }
        });
        !was_visible
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum PopupWindow {
    SidebarRight,
    SidebarLeft
}

impl PopupWindow {
    fn with(&self, callback: impl FnOnce(&Popup)) {
        APP_LOCAL.with(|app| {
            let borrow_attempt = match self {
                PopupWindow::SidebarRight => app.borrow().popup_windows.borrow().get("sidebar_right").cloned(),
                PopupWindow::SidebarLeft => app.borrow().popup_windows.borrow().get("sidebar_left").cloned()
            };

            if let Some(win) = borrow_attempt {
                callback(&win);
            }
        });
    }

    pub fn show(&self) -> bool {
        self.with(|popup| popup.show());
        true
    }

    pub fn hide(&self) -> bool {
        self.with(|popup| popup.hide());
        false
    }

    pub fn toggle(&self) -> bool {
        let mut was_visible = false;
        self.with(|popup| {
            if popup.is_visible() {
                popup.hide();
                was_visible = true;
            } else {
                popup.show();
                was_visible = false;
            }
        });
        !was_visible
    }
}

pub fn listen_for_ipc_messages() {
    ipc::listen_for_messages_local(|message| {
        match message.as_str() {
            "show_overview" => Window::Overview.show(),
            "hide_overview" => Window::Overview.hide(),
            "toggle_overview" => {
                let toggled = Window::Overview.toggle();
                if toggled {
                    let _ = ipc::client::send_message("update_overview_windows");
                }
                false
            },
            "show_session" => Window::Session.show(),
            "hide_session" => Window::Session.hide(),
            "toggle_session" => Window::Session.toggle(),
            "show_right_sidebar" => PopupWindow::SidebarRight.show(),
            "hide_right_sidebar" => PopupWindow::SidebarRight.hide(),
            "toggle_right_sidebar" => PopupWindow::SidebarRight.toggle(),
            "show_left_sidebar" => PopupWindow::SidebarLeft.show(),
            "hide_left_sidebar" => PopupWindow::SidebarLeft.hide(),
            "toggle_left_sidebar" => PopupWindow::SidebarLeft.toggle(),
            _ => false
        };
    });
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