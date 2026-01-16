use std::cell::RefCell;
use std::rc::Rc;
use gtk4::prelude::*;

use crate::singletons::notifications::{self, clear_notifications};
use crate::singletons::notifications::bus::BusEvent;
use crate::widgets::notifications::notification::{self, NotificationWidget};

pub fn new() -> gtk4::Box {
    let widgets = Rc::new(RefCell::new(Vec::new()));
    
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
        
        notifications_box = gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 0,
        },
        
        notifications_scrolled_window = gtk4::ScrolledWindow {
            set_hscrollbar_policy: gtk4::PolicyType::Never,
            set_vscrollbar_policy: gtk4::PolicyType::Automatic,
            set_vexpand: true,
            set_child: Some(&notifications_box)
        },
        
        root = gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 8,
            set_css_classes: &["notification-tab-root"],
            append: &bx,
            append: &notifications_scrolled_window,
        },
    }

    let mut receiver = notifications::subscribe();
    gtk4::glib::spawn_future_local(async move {
        while let Ok(event) = receiver.recv().await {
            let update_header = || {
                let count = notifications::NOTIFICATIONS.get()
                    .map_or(0, |notifications| notifications.read().unwrap().len());

                header_counter.set_label(&format!("{} notification{}", count, if count == 1 { "" } else { "s" }));
            };
            
            match event {
                BusEvent::NotificationAdded(_, notification) => {
                    let mut notif_widget = NotificationWidget::new(notification.clone());
                    notif_widget.set_notifications_ref(&widgets);
                    notifications_box.prepend(&notif_widget.root);
                    widgets.borrow_mut().push(notif_widget);
                    update_header();
                },
                
                BusEvent::NotificationClosed(id) => {
                    let widgets = widgets.borrow().clone();
                    for widget in &widgets {
                        if widget.notification.borrow().id == id {
                            widget.destroy(Some(notification::NotificationDismissAnimation::Right));
                        }
                    }
                    update_header();
                },
                
                BusEvent::NotificationUpdated(id, notification) => {
                    let widgets = widgets.borrow().clone();
                    for widget in &widgets {
                        if widget.notification.borrow().id == id {
                            widget.update(&notification);
                        }
                    }
                },
            
                _ => {}
            }
        }
    });

    root
}