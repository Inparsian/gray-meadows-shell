use std::sync::{Arc, Mutex};
use futures_signals::signal::SignalExt;
use gtk4::prelude::*;

use crate::singletons::hyprland;

const SHOWN_WORKSPACES: usize = 10;

#[derive(Clone)]
struct WorkspaceMask {
    pub mask: u32
}

impl WorkspaceMask {
    pub fn new() -> Self {
        Self {
            mask: 0
        }
    }

    pub fn update(&mut self) {
        let workspaces = hyprland::HYPRLAND.workspaces.get_cloned();
        let active_workspace = hyprland::HYPRLAND.active_workspace.get_cloned();
        
        self.mask = if let (Some(workspaces), Some(active_workspace)) = (workspaces, active_workspace) {
            let active_workspace_id = active_workspace.id;
            let offset = ((active_workspace_id as f64 / SHOWN_WORKSPACES as f64).floor() * SHOWN_WORKSPACES as f64) as i32;
            let mut mask = 0;

            for workspace in workspaces.iter() {
                if workspace.id > offset && workspace.id <= offset + SHOWN_WORKSPACES as i32 && workspace.windows > 0 {
                    mask |= 1 << (workspace.id - offset);
                }
            }

            mask
        } else {
            0
        };
    }
}

pub fn new() -> gtk4::Box {
    let workspace_mask: Arc<Mutex<WorkspaceMask>> = Arc::new(Mutex::new(WorkspaceMask::new()));

    relm4_macros::view! {
        workspaces_box = gtk4::Box {
            set_orientation: gtk4::Orientation::Horizontal,
            set_spacing: 1,
            set_css_classes: &["bar-widget", "bar-workspaces"],
        }
    }

    let workspaces_future = hyprland::HYPRLAND.workspaces.signal_cloned().for_each({
        let workspace_mask = workspace_mask.clone();
        move |_| {
            let mut mutex = workspace_mask.lock().unwrap();
            mutex.update();

            async {}
        }
    });

    let active_workspace_future = hyprland::HYPRLAND.active_workspace.signal_cloned().for_each({
        let workspace_mask = workspace_mask.clone();
        move |_| {
            let mut mutex = workspace_mask.lock().unwrap();
            mutex.update();

            async {}
        }
    });

    gtk4::glib::MainContext::default().spawn_local(workspaces_future);
    gtk4::glib::MainContext::default().spawn_local(active_workspace_future);

    workspaces_box
}