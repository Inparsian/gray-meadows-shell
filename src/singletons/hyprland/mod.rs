use futures_signals::signal::Mutable;
use gdk4::prelude::MonitorExt;
use once_cell::sync::Lazy;
use hyprland::{
    async_closure,
    data::{Client, Workspace, Workspaces, Monitor},
    event_listener::AsyncEventListener,
    shared::{HyprData, HyprDataActive, HyprDataActiveOptional}
};

use crate::helpers::display;

// Wrapper structs to work with Hyprland data reactively
#[derive(Default)]
pub struct Hyprland {
    pub active_client: Mutable<Option<Client>>,
    pub active_workspace: Mutable<Option<Workspace>>,
    pub workspaces: Mutable<Option<Workspaces>>,
}

pub static HYPRLAND: Lazy<Hyprland> = Lazy::new(Hyprland::default);

fn refresh_active_client() {
    let active_client = Client::get_active();
    if let Ok(active_client) = active_client {
        HYPRLAND.active_client.set(active_client);
    } else {
        HYPRLAND.active_client.set(None);
    }
}

fn refresh_active_workspace() {
    let active_workspace = Workspace::get_active();
    if let Ok(active_workspace) = active_workspace {
        HYPRLAND.active_workspace.set(Some(active_workspace));
    } else {
        HYPRLAND.active_workspace.set(None);
    }

    refresh_active_client();
}

fn refresh_workspaces() {
    let workspaces = Workspaces::get();
    if let Ok(workspaces) = workspaces {
        HYPRLAND.workspaces.set(Some(workspaces));
    } else {
        HYPRLAND.workspaces.set(None);
    }
}

pub fn activate() {
    refresh_active_client();
    refresh_active_workspace();
    refresh_workspaces();

    tokio::spawn(async move {
        let mut event_listener = AsyncEventListener::new();

        event_listener.add_window_closed_handler(async_closure! { |_| refresh_active_client() });
        event_listener.add_active_window_changed_handler(async_closure! { |_| refresh_active_client() });
        event_listener.add_float_state_changed_handler(async_closure! { |_| refresh_active_client() });
        event_listener.add_window_title_changed_handler(async_closure! { |_| refresh_active_client() });
        event_listener.add_fullscreen_state_changed_handler(async_closure! { |_| refresh_active_client() });
        event_listener.add_workspace_added_handler(async_closure! { |_| refresh_workspaces() });
        event_listener.add_workspace_deleted_handler(async_closure! { |_| refresh_workspaces() });
        event_listener.add_workspace_moved_handler(async_closure! { |_| refresh_workspaces() });
        event_listener.add_workspace_changed_handler(async_closure! { |_| refresh_active_workspace() });
        event_listener.add_active_monitor_changed_handler(async_closure! { |_| refresh_active_workspace() });

        let _ = event_listener.start_listener_async().await;
    });
}

pub fn get_active_monitor() -> Option<gdk4::Monitor> {
    let monitor = Monitor::get_active();

    if let Ok(monitor) = monitor {
        // Get the gdk4::Monitor from the display.
        let monitors = display::get_all_monitors(&gdk4::Display::default()?);

        for m in monitors {
            let geometry = m.geometry();

            if geometry.x() == monitor.x &&
                geometry.y() == monitor.y &&
                geometry.width() == monitor.width as i32 &&
                geometry.height() == monitor.height as i32
            {
                return Some(m);
            }
        }
    }

    None
}