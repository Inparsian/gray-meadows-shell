pub mod notification;

use std::rc::Rc;
use std::cell::RefCell;
use std::time::Duration;
use gtk4::prelude::*;
use gdk4::cairo::{Region, RectangleInt};
use gtk4_layer_shell::{Edge, Layer, LayerShell as _};

use crate::{APP, APP_LOCAL};
use crate::singletons::notifications::{self, wrapper::NotificationHint};
use crate::widgets::notifications::notification::NotificationDismissAnimation;
use self::notification::NotificationWidget;

const NOTIF_DISPLAY_TIMEOUT: i32 = 2500; // ms
const CRITICAL_NOTIF_DISPLAY_TIMEOUT: i32 = 5000; // ms

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

        // clamp to 300px
        let clamp = libadwaita::Clamp::new();
        clamp.set_width_request(300);
        clamp.set_maximum_size(300);
        clamp.set_unit(libadwaita::LengthUnit::Px);
        clamp.set_child(Some(&widget.root));

        self.container.prepend(&clamp);

        let timeout = if widget.notification.borrow().hints.iter().any(|hint| {
            matches!(hint, NotificationHint::Urgency(u) if *u >= 2)
        }) {
            CRITICAL_NOTIF_DISPLAY_TIMEOUT
        } else {
            NOTIF_DISPLAY_TIMEOUT
        } as u64;

        gtk4::glib::timeout_add_local_once(Duration::from_millis(timeout), {
            let widget = widget.clone();
            move || {
                widget.queue_destroy(Some(NotificationDismissAnimation::Right));
            }
        });
    }
}

pub fn listen_for_notifications() {
    use notifications::bus::BusEvent;
    let mut receiver = notifications::subscribe();

    gtk4::glib::spawn_future_local(async move {
        while let Ok(message) = receiver.recv().await {
            match message {
                BusEvent::NotificationAdded(notification) => if !APP.do_not_disturb.get() {
                    APP_LOCAL.with(move |app| {
                        for container in app.notification_containers.borrow().iter() {
                            let mut notif_widget = NotificationWidget::new(notification.clone());
                            notif_widget.set_notifications_ref(&container.widgets);
                            container.add_widget(&notif_widget);
                        }
                    });
                },

                BusEvent::NotificationClosed(id) => {
                    APP_LOCAL.with(move |app| {
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
            }
        }
    });
}