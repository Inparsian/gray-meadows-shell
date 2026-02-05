use std::cell::RefCell;
use std::rc::Rc;
use gtk::prelude::*;

use crate::services::notifications::{self, clear_notifications};
use crate::services::notifications::bus::BusEvent;
use crate::widgets::notifications::notification::{self, NotificationWidget};

pub fn new() -> gtk::Box {
    let widgets = Rc::new(RefCell::new(Vec::new()));
    
    view! {
        header_counter = gtk::Label {
            set_label: "0 notifications",
            set_css_classes: &["notification-tab-header-counter"],
            set_halign: gtk::Align::Start,
            set_hexpand: true,
            set_valign: gtk::Align::Center,
        },
        
        bx = gtk::Box {
            set_css_classes: &["notification-tab-header"],
            append: &header_counter,
            
            gtk::Button {
                set_css_classes: &["notification-tab-clear-button"],
                connect_clicked: |_| clear_notifications(),
                
                gtk::Box {
                    set_spacing: 4,
                    
                    gtk::Label {
                        set_css_classes: &["notification-tab-clear-button-icon"],
                        set_label: "clear_all",
                    },
                    
                    gtk::Label {
                        set_label: "Clear",
                    },
                }
            },
        },
        
        notifications_empty_revealer = gtk::Revealer {
            set_transition_type: gtk::RevealerTransitionType::Crossfade,
            set_transition_duration: 175,
            set_vexpand: true,
            set_reveal_child: true,
            
            gtk::Box {
                set_css_classes: &["notification-tab-empty-box"],
                set_orientation: gtk::Orientation::Vertical,
                set_valign: gtk::Align::Center,
                set_spacing: 8,
                
                gtk::Label {
                    set_css_classes: &["notification-tab-empty-icon"],
                    set_label: "notifications_none",
                },
                
                gtk::Label {
                    set_css_classes: &["notification-tab-empty-sub-label"],
                    set_label: "You're all caught up!",
                },
            }
        },
        
        notifications_box = gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 0,
        },
        
        notifications_scrolled_window = gtk::ScrolledWindow {
            set_hscrollbar_policy: gtk::PolicyType::Never,
            set_vscrollbar_policy: gtk::PolicyType::Automatic,
            set_vexpand: true,
            set_child: Some(&notifications_box)
        },
        
        notifications_view = gtk::Overlay {
            add_overlay: &notifications_scrolled_window,
            set_child: Some(&notifications_empty_revealer),
        },
        
        root = gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 8,
            set_css_classes: &["notification-tab-root"],
            append: &bx,
            append: &notifications_view,
        },
    }

    let mut receiver = notifications::subscribe();
    glib::spawn_future_local(async move {
        while let Ok(event) = receiver.recv().await {
            let update_view = || {
                let count = notifications::NOTIFICATIONS.get()
                    .map_or(0, |notifications| notifications.read().unwrap().len());

                header_counter.set_label(&format!("{} notification{}", count, if count == 1 { "" } else { "s" }));
                notifications_empty_revealer.set_reveal_child(count < 1);
            };
            
            match event {
                BusEvent::NotificationAdded(notification) => {
                    let mut notif_widget = NotificationWidget::new(notification.clone());
                    notif_widget.set_notifications_ref(&widgets);
                    notifications_box.prepend(&notif_widget.root);
                    widgets.borrow_mut().push(notif_widget);
                    update_view();
                },
                
                BusEvent::NotificationClosed(id) => {
                    let widgets = widgets.borrow().clone();
                    for widget in &widgets {
                        if widget.notification.borrow().id == id {
                            widget.destroy(Some(notification::NotificationDismissAnimation::Right));
                        }
                    }
                    update_view();
                },
                
                BusEvent::NotificationUpdated(id, notification) => {
                    let widgets = widgets.borrow().clone();
                    for widget in &widgets {
                        if widget.notification.borrow().id == id {
                            widget.update(&notification);
                        }
                    }
                },
            }
        }
    });

    root
}