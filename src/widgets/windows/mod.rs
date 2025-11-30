pub mod popup;
pub mod fullscreen;
pub mod overview;
pub mod session;
pub mod sidebar_left;
pub mod sidebar_right;

use std::any::Any;

use crate::{APP_LOCAL, ipc};

pub trait GmsWindow: Any {
    fn show(&self);
    fn hide(&self);
    fn toggle(&self) -> bool;
    fn is_visible(&self) -> bool;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl dyn GmsWindow {
    pub fn downcast_ref<T: GmsWindow>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }

    pub fn downcast_mut<T: GmsWindow>(&mut self) -> Option<&mut T> {
        self.as_any_mut().downcast_mut::<T>()
    }
}

fn with(window: &str, callback: impl FnOnce(&Box<dyn GmsWindow>)) {
    APP_LOCAL.with(|app| {
        let app = app.borrow();
        let windows = app.windows.borrow();
        let borrow_attempt = windows.get(window);

        if let Some(win) = borrow_attempt {
            callback(win);
        }
    });
}

pub fn show(window: &str) {
    with(window, |win| win.show());
}

pub fn hide(window: &str) {
    with(window, |win| win.hide());
}

pub fn toggle(window: &str) -> bool {
    let mut was_visible = false;
    with(window, |win| {
        was_visible = win.toggle();
    });
    was_visible
}

pub fn hide_all_popups() {
    APP_LOCAL.with(|app| {
        for window in app.borrow().windows.borrow().values() {
            if window.downcast_ref::<popup::PopupWindow>().is_some() {
                window.hide();
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