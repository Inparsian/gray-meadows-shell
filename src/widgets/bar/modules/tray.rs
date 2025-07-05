use gtk4::prelude::*;

use crate::singletons::tray::{bus::BusEvent, get_item, icon::make_icon_pixbuf, subscribe, wrapper::sn_item::StatusNotifierItem};

#[allow(dead_code)]
#[derive(Default, Clone)]
struct SystemTrayItem {
    pub item: StatusNotifierItem,
    pub widget: Option<gtk4::Image>,
    popover_menu: Option<gtk4::PopoverMenu>
}

#[allow(dead_code)]
impl SystemTrayItem {
    pub fn new(item: StatusNotifierItem) -> Self {
        Self {
            item,
            ..Self::default()
        }
    }

    pub fn build_widget(&mut self) {
        relm4_macros::view! {
            new_widget = gtk4::Image {
                set_css_classes: &["bar-tray-item"],
                set_pixel_size: 14
            }
        };

        if let Some(pixbuf) = make_icon_pixbuf(Some(&self.item.icon_pixmap)) {
            new_widget.set_from_pixbuf(Some(&pixbuf));
        } else {
            new_widget.set_icon_name(Some("emote-heart"));
        }

        // Set the widget
        self.widget = Some(new_widget);
    }

    pub fn update(&mut self, member: String) {
        println!("Updating item: {} with member: {}", self.item.service, member);
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

    fn add_item(&mut self, item: StatusNotifierItem) {
        let mut item = SystemTrayItem::new(item);
        item.build_widget();

        if let Some(ref widget) = item.widget {
            self.box_.append(widget);
        }

        self.items.push(item);
    }

    fn update_item(&mut self, service: String, member: String) {
        if let Some(item) = self.items.iter_mut().find(|i| i.item.service == service) {
            item.update(member);
        }
    }

    fn remove_item(&mut self, service: String) {
        if let Some(pos) = self.items.iter().position(|i| i.item.service == service) {
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
        if let Some(items) = crate::singletons::tray::ITEMS.get() {
            for item in items.lock().unwrap().iter() {
                println!("[missed] Item registered: {}", item.service);
                tray.add_item(item.clone());
            }
        } else {
            eprintln!("Failed to fetch current tray items.");
        }

        while let Ok(event) = subscribe().recv().await {
            match event {
                BusEvent::ItemRegistered(item) => {
                    println!("Item registered: {}", item.service);
                    tray.add_item(item);
                },

                BusEvent::ItemUpdated(member, item) => {
                    println!("Item updated: {} - {}", item.service, member);
                    tray.update_item(item.service, member);
                },

                BusEvent::ItemUnregistered(item) => {
                    println!("Item unregistered: {}", item.service);
                    tray.remove_item(item.service);
                },

                _ => {}
            }
        }
    });

    widget
}