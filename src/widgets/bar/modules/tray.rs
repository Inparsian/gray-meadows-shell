use gtk4::prelude::*;

use crate::singletons::tray::{TrayEvent, subscribe};

#[allow(dead_code)]
#[derive(Default, Clone)]
struct SystemTrayItem {
    pub owner: String,
    pub widget: Option<gtk4::Label>,
    popover_menu: Option<gtk4::PopoverMenu>
}

#[allow(dead_code)]
impl SystemTrayItem {
    pub fn new(owner: String) -> Self {
        Self {
            owner,
            ..Self::default()
        }
    }

    pub fn build_widget(&mut self) {
        // Create a widget for the system tray item
        let widget = gtk4::Label::new(Some(&self.owner));

        // Set the widget
        self.widget = Some(widget);
    }

    pub fn update(&mut self, member: String) {
        println!("Updating item: {} with member: {}", self.owner, member);
    }
}

#[allow(dead_code)]
#[derive(Clone)]
struct SystemTray {
    box_: gtk4::Box,
    items: Vec<SystemTrayItem>,
}

impl SystemTray {
    fn new() -> Self {
        let box_ = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        box_.set_css_classes(&["bar-widget", "bar-tray"]);
        box_.set_hexpand(false);

        let items = Vec::new();

        Self {
            box_,
            items,
        }
    }

    fn add_item(&mut self, service: String) {
        let mut item = SystemTrayItem::new(service);
        item.build_widget();

        if let Some(ref widget) = item.widget {
            self.box_.append(widget);
        }

        self.items.push(item);
    }

    fn update_item(&mut self, service: String, member: String) {
        if let Some(item) = self.items.iter_mut().find(|i| i.owner == service) {
            item.update(member);
        }
    }

    fn remove_item(&mut self, service: String) {
        if let Some(pos) = self.items.iter().position(|i| i.owner == service) {
            let item = self.items.remove(pos);
            if let Some(widget) = item.widget {
                self.box_.remove(&widget);
            }
        }
    }

    fn get_widget(&self) -> gtk4::Box {
        self.box_.clone()
    }
}

pub fn new() -> gtk4::Box {
    let mut tray = SystemTray::new();
    let widget = tray.get_widget();

    // Watch for tray events
    gtk4::glib::spawn_future_local(async move {
        // We may have missed some items that were registered before we start listening.
        // Fetch the current items and register them.
        if let Ok(items) = crate::singletons::tray::get_items().await {
            for service in items {
                let item_path = format!("{}/StatusNotifierItem", service);
                println!("[missed] Item registered: {}", item_path);
                tray.add_item(item_path);
            }
        } else {
            eprintln!("Failed to fetch current tray items.");
        }

        let mut receiver = subscribe().await;
        while let Ok(event) = receiver.recv().await {
            match event {
                TrayEvent::Register(service) => {
                    println!("Item registered: {}", service);
                    tray.add_item(service);
                },

                TrayEvent::Update(service, member) => {
                    println!("Item updated: {} - {:?}", service, member);
                    tray.update_item(service, member);
                },

                TrayEvent::Unregister(service) => {
                    println!("Item unregistered: {}", service);
                    tray.remove_item(service);
                }
            }
        }
    });

    widget
}