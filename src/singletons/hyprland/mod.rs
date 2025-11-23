use std::sync::LazyLock;
use futures_signals::signal::Mutable;
use gdk4::prelude::MonitorExt;
use hyprland::{
    data::{Client, Monitor, Workspace, Workspaces},
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
    pub submap: Mutable<Option<String>>
}

pub static HYPRLAND: LazyLock<Hyprland> = LazyLock::new(Hyprland::default);

fn refresh_active_client() {
    HYPRLAND.active_client.set(Client::get_active().ok().unwrap_or(None));
}

fn refresh_active_workspace() {
    HYPRLAND.active_workspace.set(Workspace::get_active().ok());
    refresh_active_client();
}

fn refresh_workspaces() {
    HYPRLAND.workspaces.set(Workspaces::get().ok());
}

fn refresh_submap(new_submap: Option<String>) {
    if let Some(submap) = new_submap {
        HYPRLAND.submap.set(Some(submap));
    } else {
        // hyprland-rs currently does not implement the 'submap' command for
        // fetching the current submap, this must be fetched via a direct hyprctl
        // call.
        if let Ok(output) = std::process::Command::new("hyprctl")
            .arg("submap")
            .output()
        {
            if output.status.success() {
                let submap = String::from_utf8_lossy(&output.stdout).trim().to_owned();
                if !submap.is_empty() && submap != "default" {
                    HYPRLAND.submap.set(Some(submap));
                }
            }
        }
    }
}

pub fn activate() {
    refresh_active_client();
    refresh_active_workspace();
    refresh_workspaces();
    refresh_submap(None);

    tokio::spawn(async move {
        let mut event_listener = AsyncEventListener::new();

        event_listener.add_window_closed_handler(|_| Box::pin(async { refresh_active_client() }));
        event_listener.add_active_window_changed_handler(|_| Box::pin(async { refresh_active_client() }));
        event_listener.add_float_state_changed_handler(|_| Box::pin(async { refresh_active_client() }));
        event_listener.add_window_title_changed_handler(|_| Box::pin(async { refresh_active_client() }));
        event_listener.add_fullscreen_state_changed_handler(|_| Box::pin(async { refresh_active_client() }));
        event_listener.add_workspace_added_handler(|_| Box::pin(async { refresh_workspaces() }));
        event_listener.add_workspace_deleted_handler(|_| Box::pin(async { refresh_workspaces() }));
        event_listener.add_workspace_moved_handler(|_| Box::pin(async { refresh_workspaces() }));
        event_listener.add_workspace_changed_handler(|_| Box::pin(async { refresh_active_workspace() }));
        event_listener.add_active_monitor_changed_handler(|_| Box::pin(async { refresh_active_workspace() }));
        event_listener.add_sub_map_changed_handler(|new_submap| Box::pin(async { refresh_submap(Some(new_submap)) }));
        
        let _ = event_listener.start_listener_async().await;
    });
}

pub fn get_active_monitor() -> Option<gdk4::Monitor> {
    if let Ok(monitor) = Monitor::get_active() {
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