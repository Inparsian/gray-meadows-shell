use gtk4::prelude::*;

use crate::singletons::notifications;
use crate::singletons::notifications::bus::BusEvent;

pub fn new() -> gtk4::Box {
    let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    root.set_css_classes(&["notification-tab-root"]);

    let bx = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    bx.set_css_classes(&["notification-tab-header"]);

    let header_counter = gtk4::Label::new(Some("0 notifications"));
    header_counter.set_css_classes(&["notification-tab-header-counter"]);
    header_counter.set_halign(gtk4::Align::Start);
    header_counter.set_valign(gtk4::Align::Center);
    bx.append(&header_counter);

    let mut receiver = notifications::subscribe();
    gtk4::glib::spawn_future_local(async move {
        while let Ok(event) = receiver.recv().await {
            match event {
                BusEvent::NotificationAdded { .. } |
                BusEvent::NotificationClosed { .. } => {
                    let count = notifications::NOTIFICATIONS.get()
                        .map_or(0, |notifications| notifications.read().unwrap().len());

                    header_counter.set_label(&format!("{} notification{}", count, if count == 1 { "" } else { "s" }));
                },
            
                _ => {}
            }
        }
    });

    root.append(&bx);
    root
}