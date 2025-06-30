use futures_signals::signal_vec::SignalVecExt;
use gtk4::prelude::*;

use crate::singletons::tray;

#[derive(Clone)]
struct SystemTrayItem {
    pub owner: String,
    pub widget: Option<gtk4::Image>
}

impl SystemTrayItem {
    pub fn new(owner: String) -> Self {
        Self { owner, widget: None }
    }

    pub fn build(&mut self) {
        println!("Building SystemTrayItem for owner: {}", self.owner);
        if let Some(item) = tray::get_tray_item(&self.owner) {            
            relm4_macros::view! {
                new_widget = gtk4::Image {
                    set_css_classes: &["bar-tray-item"],
                    set_from_pixbuf: Some(&tray::make_icon_pixbuf(item)),
                    set_pixel_size: 14
                }
            };

            self.widget = Some(new_widget);
        }
    }

    pub fn update(&mut self) {
        println!("Updating SystemTrayItem for owner: {}", self.owner);
        if let Some(widget) = &self.widget {
            if let Some(item) = tray::get_tray_item(&self.owner) {
                widget.set_from_pixbuf(Some(&tray::make_icon_pixbuf(item)));
            }
        }
    }
}

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

        let mut items = Vec::new();

        // add existing tray items
        for item in tray::TRAY_ITEMS.lock_ref().iter() {
            let mut tray_item = SystemTrayItem::new(item.0.clone());
            tray_item.build();
            if let Some(ref widget) = tray_item.widget {
                box_.append(widget);
            }
            items.push(tray_item);
        }

        Self {
            box_,
            items,
        }
    }

    fn add_item(&mut self, owner: String) {
        // assert that the item does not already exist
        if self.items.iter().any(|item| item.owner == owner) {
            println!("Item with owner '{}' already exists, skipping addition.", owner);
            return;
        } else {
            println!("Adding SystemTrayItem for owner: {}", owner);
        }

        let mut item = SystemTrayItem::new(owner);

        item.build();
        if let Some(ref widget) = item.widget {
            self.box_.append(widget);
        }

        self.items.push(item);
    }

    fn remove_item(&mut self, owner: &str) {
        if let Some(pos) = self.items.iter().position(|item| item.owner == owner) {
            let item = self.items.remove(pos);

            if let Some(widget) = item.widget {
                self.box_.remove(&widget);
            }
        }
    }

    fn remove_item_index(&mut self, index: &usize) {
        if let Some(owner) = self.items.get(*index).map(|item| item.owner.clone()) {
            self.remove_item(&owner);
        }
    }

    fn pop_item(&mut self) {
        if let Some(item) = self.items.pop() {
            if let Some(widget) = item.widget {
                self.box_.remove(&widget);
            }
        }
    }

    fn update_item(&mut self, owner: &str) {
        if let Some(item) = self.items.iter_mut().find(|item| item.owner == owner) {
            item.update();
        }
    }

    fn get_widget(&self) -> gtk4::Box {
        self.box_.clone()
    }
}

pub fn new() -> gtk4::Box {
    let mut tray = SystemTray::new();
    let widget = tray.get_widget();

    // Subscribe to tray item changes
    let tray_items = tray::TRAY_ITEMS.clone();
    let tray_items_future = tray_items.signal_vec_cloned().for_each(move |diff| {
        match diff {
            futures_signals::signal_vec::VecDiff::RemoveAt { index } => {
                tray.remove_item_index(&index);
            }

            futures_signals::signal_vec::VecDiff::InsertAt { index: _, value } => {
                tray.add_item(value.0.clone());
            }

            futures_signals::signal_vec::VecDiff::Push { value } => {
                tray.add_item(value.0.clone());
            }

            futures_signals::signal_vec::VecDiff::Pop {} => {
                tray.pop_item();
            }

            futures_signals::signal_vec::VecDiff::UpdateAt { index: _, value } => {
                tray.update_item(&value.0);
            }

            _ => {}
        }

        async {}
    });

    gtk4::glib::MainContext::default().spawn_local(tray_items_future);

    widget
}