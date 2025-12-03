use std::sync::{Mutex, LazyLock};
use futures_signals::signal::SignalExt;
use gtk4::prelude::*;
use ::hyprland::dispatch;
use ::hyprland::dispatch::WorkspaceIdentifierWithSpecial;

use crate::{
    singletons::hyprland,
    helpers::{scss, gesture},
    widgets::bar::wrapper::SimpleBarModuleWrapper
};

const SHOWN_WORKSPACES: usize = 10;
const WORKSPACE_WIDTH: f64 = 13.0;
const WORKSPACE_HEIGHT: f64 = 13.0;
const WORKSPACE_Y: f64 = 5.0;
const WORKSPACE_PADDING: f64 = 1.0;

static WORKSPACE_MASK: LazyLock<Mutex<WorkspaceMask>> = LazyLock::new(|| Mutex::new(WorkspaceMask::default()));

#[derive(Clone, Default)]
struct WorkspaceMask {
    pub mask: u32
}

impl WorkspaceMask {
    pub fn update(&mut self) {
        let workspaces = hyprland::HYPRLAND.workspaces.get_cloned();
        let active_workspace = hyprland::HYPRLAND.active_workspace.get_cloned();
        
        self.mask = if let (Some(workspaces), Some(active_workspace)) = (workspaces, active_workspace) {
            let offset = (((active_workspace.id - 1) as f64 / SHOWN_WORKSPACES as f64).floor() * SHOWN_WORKSPACES as f64) as i32;
            let mut mask = 0;

            for workspace in &workspaces {
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

    view! {
        workspaces_click_gesture = gesture::on_primary_down(|_, x, _| {
            let ws = (((x - 5.0 - WORKSPACE_PADDING) / (WORKSPACE_WIDTH + WORKSPACE_PADDING)) + 1.0).floor() as i32;

            if ws >= 1 && ws <= SHOWN_WORKSPACES as i32 {
                if let Some(active_workspace) = hyprland::HYPRLAND.active_workspace.get_cloned() {
                    let clamped = ((active_workspace.id - 1) / 10) * 10 + ws;
                    let _ = dispatch!(Workspace, WorkspaceIdentifierWithSpecial::Id(clamped));
                }
            }
        }),

        workspaces_scroll_gesture = gesture::on_vertical_scroll(|dy| {
            let delta = if dy < 0.0 { -1 } else { 1 };
            let _ = dispatch!(Workspace, WorkspaceIdentifierWithSpecial::RelativeMonitorIncludingEmpty(delta));
        }),

        workspaces_drawing_area = gtk4::DrawingArea {
            set_css_classes: &["bar-workspaces-drawingarea"],

            set_draw_func: {
                move |area, cr, _, _| {
                    area.set_size_request((SHOWN_WORKSPACES as i32 + 1) * WORKSPACE_WIDTH as i32, 16);

                    let active_ws: f64 = area.pango_context().font_description()
                        .map_or(1.0, |desc| desc.size() as f64 / gtk4::pango::SCALE as f64);

                    // draw workspace squares
                    for i in 0..SHOWN_WORKSPACES+1 {
                        let workspace_x = (i as f64 - 1.0).mul_add(WORKSPACE_WIDTH + WORKSPACE_PADDING, WORKSPACE_PADDING);
                        let color_variable_name = if WORKSPACE_MASK.lock().unwrap().mask & (1 << i) != 0 {
                            "foreground-color-primary"
                        } else {
                            "foreground-color-quinary"
                        };

                        if let Some(color) = scss::get_color(color_variable_name) {
                            cr.set_source_rgba(
                                color.red as f64 / 255.0,
                                color.green as f64 / 255.0,
                                color.blue as f64 / 255.0,
                                color.alpha as f64 / 255.0
                            );

                            cr.rectangle(
                                ((workspace_x + (WORKSPACE_WIDTH / 4.0)) + 2.0).ceil(),
                                ((WORKSPACE_Y + (WORKSPACE_HEIGHT / 4.0)) + 1.0).ceil(),
                                ((WORKSPACE_WIDTH / 2.0) - 4.0).ceil(),
                                ((WORKSPACE_HEIGHT / 2.0) - 4.0).ceil()
                            );

                            let _ = cr.fill();
                        }
                    }

                    // draw active workspace
                    if let Some(color) = scss::get_color("foreground-color-primary") {
                        let active_workspace_x = (active_ws - 1.0).mul_add(WORKSPACE_WIDTH + WORKSPACE_PADDING, WORKSPACE_PADDING) + 1.0;

                        cr.set_source_rgba(
                            color.red as f64 / 255.0,
                            color.green as f64 / 255.0,
                            color.blue as f64 / 255.0,
                            color.alpha as f64 / 255.0
                        );

                        cr.rectangle(
                            ((active_workspace_x + (WORKSPACE_WIDTH / 4.0)) - 1.0).ceil(),
                            ((WORKSPACE_Y + (WORKSPACE_HEIGHT / 4.0)) - 1.0).ceil(),
                            (WORKSPACE_WIDTH / 2.0).ceil(),
                            (WORKSPACE_HEIGHT / 2.0).ceil()
                        );

                        let _ = cr.fill();
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

    gtk4::glib::spawn_future_local({
        let workspaces_drawing_area = workspaces_drawing_area.clone();
        signal_cloned!(hyprland::HYPRLAND.workspaces, (_) {
            WORKSPACE_MASK.lock().unwrap().update();
            workspaces_drawing_area.queue_draw();
        })
    });

    gtk4::glib::spawn_future_local(signal_cloned!(hyprland::HYPRLAND.active_workspace, (active) {
        WORKSPACE_MASK.lock().unwrap().update();
            
        if let Some(active) = active {
            style_provider.load_from_data(&format!(
                ".bar-workspaces-drawingarea {{ font-size: {}px; }}",
                ((active.id - 1) % SHOWN_WORKSPACES as i32) + 1
            ));
        }

        workspaces_drawing_area.queue_draw();
    }));

    SimpleBarModuleWrapper::new(&workspaces_box)
        .add_controller(workspaces_click_gesture)
        .add_controller(workspaces_scroll_gesture)
        .get_widget()
}