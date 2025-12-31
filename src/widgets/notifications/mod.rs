pub mod notification;

use std::rc::Rc;
use std::cell::RefCell;
use gtk4::prelude::*;
use gdk4::cairo::{Region, RectangleInt};
use gtk4_layer_shell::{Edge, Layer, LayerShell as _};

use crate::APP_LOCAL;
use crate::singletons::notifications;
use self::notification::NotificationWidget;

#[derive(Clone)]
pub struct NotificationsWindow {
    pub widgets: Rc<RefCell<Vec<NotificationWidget>>>,
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
            widgets: Rc::new(RefCell::new(Vec::new())),
            window,
            container,
        }
    }

    pub fn add_widget(&self, widget: &NotificationWidget) {
        self.widgets.borrow_mut().push(widget.clone());
        self.container.prepend(&widget.root);
    }
}

pub fn listen_for_notifications() {
    use notifications::bus::BusEvent;
    let mut receiver = notifications::subscribe();

    gtk4::glib::spawn_future_local(async move {
        while let Ok(message) = receiver.recv().await {
            match message {
                BusEvent::NotificationAdded(_, notification) => {
                    APP_LOCAL.with(move |app| {
                        let app = app.borrow();
                        for container in app.notification_containers.borrow().iter() {
                            let mut notif_widget = NotificationWidget::new(notification.clone());
                            notif_widget.set_parent(Rc::new(container.clone()));
                            container.add_widget(&notif_widget);
                        }
                    });
                },

                BusEvent::NotificationClosed(id) => {
                    APP_LOCAL.with(move |app| {
                        let app = app.borrow();
                        for container in app.notification_containers.borrow().iter() {
                            let widgets = container.widgets.borrow().clone();
                            for widget in &widgets {
                                if widget.notification.borrow().id == id {
                                    widget.destroy(Some(notification::NotificationDismissAnimation::Right));
                                }
                            }
                        }
                    });
                },

                BusEvent::NotificationUpdated(id, notification) => {
                    APP_LOCAL.with(move |app| {
                        let app = app.borrow();
                        for container in app.notification_containers.borrow().iter() {
                            let widgets = container.widgets.borrow().clone();
                            for widget in &widgets {
                                if widget.notification.borrow().id == id {
                                    widget.update(&notification);
                                }
                            }
                        }
                    });
                },

                _ => {},
            }
        }
    });
}