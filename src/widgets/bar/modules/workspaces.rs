use std::sync::{Arc, Mutex};
use futures_signals::signal::SignalExt;
use gtk4::prelude::*;

use crate::singletons::hyprland;
use crate::helpers::scss;

const SHOWN_WORKSPACES: usize = 10;
const WORKSPACE_WIDTH: f64 = 13.0;
const WORKSPACE_HEIGHT: f64 = 13.0;
const WORKSPACE_Y: f64 = 5.0;
const WORKSPACE_PADDING: f64 = 1.0;

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
            let offset = (((active_workspace.id - 1) as f64 / SHOWN_WORKSPACES as f64).floor() * SHOWN_WORKSPACES as f64) as i32;
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
    let style_provider = gtk4::CssProvider::new();
    let workspace_mask: Arc<Mutex<WorkspaceMask>> = Arc::new(Mutex::new(WorkspaceMask::new()));

    relm4_macros::view! {
        workspaces_drawing_area = gtk4::DrawingArea {
            set_css_classes: &["bar-workspaces-drawingarea"],
            set_draw_func: {
                let workspace_mask = workspace_mask.clone();
                move |area, cr, _, _| {
                    area.set_size_request((SHOWN_WORKSPACES as i32 + 1) * WORKSPACE_WIDTH as i32, 16);

                    let active_ws: f64 = if let Some(font_desc) = area.pango_context().font_description() {
                        font_desc.size() as f64 / gtk4::pango::SCALE as f64
                    } else {
                        1.0 // fallback to workspace 1
                    };

                    // draw workspace squares
                    for i in 0..SHOWN_WORKSPACES+1 {
                        let workspace_x = (i as f64 - 1.0) * (WORKSPACE_WIDTH + WORKSPACE_PADDING) + WORKSPACE_PADDING;
                        let color_variable_name = if workspace_mask.lock().unwrap().mask & (1 << i) != 0 {
                            "foreground-color-primary"
                        } else {
                            "foreground-color-third"
                        };

                        if let Some(color) = scss::get_color(color_variable_name) {
                            cr.set_source_rgba(color.red, color.green, color.blue, color.alpha);

                            cr.rectangle(
                                ((workspace_x + (WORKSPACE_WIDTH / 4.0)) + 2.0).ceil(),
                                ((WORKSPACE_Y + (WORKSPACE_HEIGHT / 4.0)) + 1.0).ceil(),
                                ((WORKSPACE_WIDTH / 2.0) - 4.0).ceil(),
                                ((WORKSPACE_HEIGHT / 2.0) - 4.0).ceil()
                            );

                            let _ = cr.fill();
                        }
                    }
                }
            }
        },

        workspaces_box = gtk4::Box {
            set_orientation: gtk4::Orientation::Horizontal,
            set_spacing: 0,
            set_css_classes: &["bar-widget", "bar-workspaces"],

            append: &workspaces_drawing_area
        }
    }

    workspaces_drawing_area.style_context().add_provider(&style_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

    let workspaces_future = hyprland::HYPRLAND.workspaces.signal_cloned().for_each({
        let workspace_mask = workspace_mask.clone();
        let workspaces_drawing_area = workspaces_drawing_area.clone();
        move |_| {
            let mut mutex = workspace_mask.lock().unwrap();
            mutex.update();
            workspaces_drawing_area.queue_draw();

            async {}
        }
    });

    let active_workspace_future = hyprland::HYPRLAND.active_workspace.signal_cloned().for_each({
        let workspace_mask = workspace_mask.clone();
        let workspaces_drawing_area = workspaces_drawing_area.clone();
        move |active| {
            let mut mutex = workspace_mask.lock().unwrap();
            mutex.update();
            
            if let Some(active) = active {
                style_provider.load_from_data(&format!(
                    ".bar-workspaces-drawingarea {{ font-size: {}px; }}",
                    ((active.id - 1) % SHOWN_WORKSPACES as i32) + 1
                ));
            }

            workspaces_drawing_area.queue_draw();

            async {}
        }
    });

    gtk4::glib::MainContext::default().spawn_local(workspaces_future);
    gtk4::glib::MainContext::default().spawn_local(active_workspace_future);

    workspaces_box
}