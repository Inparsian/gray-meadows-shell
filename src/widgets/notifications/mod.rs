pub mod notification;

use gtk4::prelude::*;
use gdk4::cairo::{Region, RectangleInt};
use gtk4_layer_shell::{Edge, Layer, LayerShell as _};

use crate::APP_LOCAL;
use crate::singletons::notifications;

#[derive(Clone)]
pub struct NotificationsWindow {
    pub window: gtk4::ApplicationWindow,
    pub container: gtk4::Box,
}

impl NotificationsWindow {
    pub fn new(application: &libadwaita::Application, monitor: &gdk4::Monitor) -> Self {
        view! {
            container = gtk4::Box {
                set_css_classes: &["notifications-container"],
                set_orientation: gtk4::Orientation::Vertical,
                set_spacing: 0,
                set_halign: gtk4::Align::End,
                set_valign: gtk4::Align::Start,
                set_overflow: gtk4::Overflow::Visible,
            },

            window = gtk4::ApplicationWindow {
                set_css_classes: &["notifications-window"],
                set_application: Some(application),
                init_layer_shell: (),
                set_monitor: Some(monitor),
                set_layer: Layer::Overlay,
                set_anchor: (Edge::Top, true),
                set_anchor: (Edge::Bottom, true),
                set_anchor: (Edge::Left, true),
                set_anchor: (Edge::Right, true),
                set_namespace: Some("gms-notifications"),
                set_child: Some(&container),
            }
        }

        // Update the input region every frame to match the size of the window's child
        window.add_tick_callback(move |window, _| {
            if let Some(child) = window.child() {
                let allocation = child.allocation();
                let region = Region::create_rectangle(&RectangleInt::new(
                    allocation.x(),
                    allocation.y(),
                    allocation.width(),
                    allocation.height(),
                ));

                if let Some(surface) = window.native().and_then(|n| n.surface()) {
                    surface.set_input_region(&region);
                }
            }

            gdk4::glib::ControlFlow::Continue
        });

        NotificationsWindow {
            window,
            container,
        }
    }
}

pub fn listen_for_notifications() {
    let mut receiver = notifications::subscribe();

    gtk4::glib::spawn_future_local(async move {
        while let Ok(message) = receiver.recv().await {
            if let notifications::bus::BusEvent::NotificationAdded(_, notification) = message {
                APP_LOCAL.with(move |app| {
                    let app = app.borrow();
                    for container in app.notification_containers.borrow().iter() {
                        let notif_widget = notification::NotificationWidget::new(notification.clone());
                        container.container.prepend(&notif_widget.root);
                    }
                });
            }
        }
    });
}