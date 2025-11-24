use gtk4::prelude::*;
use gtk4_layer_shell::LayerShell;

use crate::{APP_LOCAL, bind_events, ipc, singletons::hyprland};

pub enum Window {
    Overview,
    Session,
    SidebarLeft,
    SidebarRight,
}

impl Window {
    fn with(&self, callback: impl FnOnce(&gtk4::ApplicationWindow)) {
        APP_LOCAL.with(|app| {
            let borrow_attempt = match self {
                Window::Overview => app.borrow().overview_window.borrow().as_ref().cloned(),
                Window::Session => app.borrow().session_window.borrow().as_ref().cloned(),
                Window::SidebarLeft => app.borrow().sidebar_left_window.borrow().as_ref().cloned(),
                Window::SidebarRight => app.borrow().sidebar_right_window.borrow().as_ref().cloned(),
            };

            if let Some(win) = borrow_attempt {
                callback(&win);
            }
        });
    }

    pub fn is_focused(&self) -> bool {
        let mut focused = false;
        self.with(|win| focused = win.is_active());
        focused
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

pub fn handle_mouse_events() {
    bind_events::listen_for_mouse_events({
        move |event| if let bind_events::MouseEvent::Release(button) = event {
            if button == bind_events::MouseButton::Left {
                gtk4::glib::MainContext::default().invoke(move || {
                    for window in [Window::Overview, Window::Session, Window::SidebarLeft, Window::SidebarRight] {
                        if !window.is_focused() {
                            window.hide();
                        }
                    }
                });
            }
        }
    });
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