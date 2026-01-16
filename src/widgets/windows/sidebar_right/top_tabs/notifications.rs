use gtk4::prelude::*;

use crate::singletons::notifications::{self, clear_notifications};
use crate::singletons::notifications::bus::BusEvent;

pub fn new() -> gtk4::Box {
    view! {
        header_counter = gtk4::Label {
            set_label: "0 notifications",
            set_css_classes: &["notification-tab-header-counter"],
            set_halign: gtk4::Align::Start,
            set_hexpand: true,
            set_valign: gtk4::Align::Center,
        },
        
        bx = gtk4::Box {
            set_css_classes: &["notification-tab-header"],
            append: &header_counter,
            
            gtk4::Button {
                set_css_classes: &["notification-tab-clear-button"],
                connect_clicked: |_| clear_notifications(),
                
                gtk4::Box {
                    set_spacing: 4,
                    
                    gtk4::Label {
                        set_css_classes: &["notification-tab-clear-button-icon"],
                        set_label: "clear_all",
                    },
                    
                    gtk4::Label {
                        set_label: "Clear",
                    },
                }
            },
        },
        
        root = gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_css_classes: &["notification-tab-root"],
            append: &bx,
        },
    }

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

    root
}