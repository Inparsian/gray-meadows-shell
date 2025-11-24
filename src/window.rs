use gtk4::prelude::*;
use gtk4_layer_shell::LayerShell;

use crate::{APP_LOCAL, singletons::hyprland, ipc};

pub enum Window {
    Overview,
    Session,
    SidebarLeft,
    SidebarRight,
}

impl Window {
    pub fn show(self) -> bool {
        gtk4::glib::MainContext::default().invoke(move || {
            APP_LOCAL.with(|app| {
                let monitor = hyprland::get_active_monitor();
                let borrow_attempt = match self {
                    Window::Overview => app.borrow().overview_window.borrow().as_ref().cloned(),
                    Window::Session => app.borrow().session_window.borrow().as_ref().cloned(),
                    Window::SidebarLeft => app.borrow().sidebar_left_window.borrow().as_ref().cloned(),
                    Window::SidebarRight => app.borrow().sidebar_right_window.borrow().as_ref().cloned(),
                };

                if let Some(win) = borrow_attempt {
                    win.set_monitor(monitor.as_ref());
                    win.show();
                }
            });
        });
        true
    }

    pub fn hide(self) -> bool {
        gtk4::glib::MainContext::default().invoke(move || {
            APP_LOCAL.with(|app| {
                let borrow_attempt = match self {
                    Window::Overview => app.borrow().overview_window.borrow().as_ref().cloned(),
                    Window::Session => app.borrow().session_window.borrow().as_ref().cloned(),
                    Window::SidebarLeft => app.borrow().sidebar_left_window.borrow().as_ref().cloned(),
                    Window::SidebarRight => app.borrow().sidebar_right_window.borrow().as_ref().cloned(),
                };

                if let Some(win) = borrow_attempt {
                    win.hide();
                }
            });
        });
        false
    }

    pub fn toggle(self) -> bool {
        let mut was_visible = false;
        gtk4::glib::MainContext::default().invoke(move || {
            APP_LOCAL.with(|app| {
                let monitor = hyprland::get_active_monitor();
                let borrow_attempt = match self {
                    Window::Overview => app.borrow().overview_window.borrow().as_ref().cloned(),
                    Window::Session => app.borrow().session_window.borrow().as_ref().cloned(),
                    Window::SidebarLeft => app.borrow().sidebar_left_window.borrow().as_ref().cloned(),
                    Window::SidebarRight => app.borrow().sidebar_right_window.borrow().as_ref().cloned(),
                };

                if let Some(win) = borrow_attempt {
                    if win.is_visible() {
                        win.hide();
                        was_visible = true;
                    } else {
                        win.set_monitor(monitor.as_ref());
                        win.show();
                        was_visible = false;
                    }
                }
            });
        });
        !was_visible
    }
}

pub fn listen_for_ipc_messages() {
    ipc::listen_for_messages(|message| {
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
            "show_right_sidebar" => Window::SidebarRight.show(),
            "hide_right_sidebar" => Window::SidebarRight.hide(),
            "toggle_right_sidebar" => Window::SidebarRight.toggle(),
            "show_left_sidebar" => Window::SidebarLeft.show(),
            "hide_left_sidebar" => Window::SidebarLeft.hide(),
            "toggle_left_sidebar" => Window::SidebarLeft.toggle(),
            _ => false
        };
    });
}