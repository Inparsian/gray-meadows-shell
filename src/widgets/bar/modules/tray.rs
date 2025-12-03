use gdk4::gio;
use gtk4::prelude::*;

use crate::{
    helpers::gesture,
    singletons::{apps::pixbuf::get_pixbuf_or_fallback, tray::{self, bus::BusEvent, icon::make_icon_pixbuf, subscribe, tray_menu, wrapper::{dbus_menu::Menu, sn_item::StatusNotifierItem}}},
    widgets::bar::wrapper::SimpleBarModuleWrapper
};

#[derive(Default, Clone)]
struct SystemTrayItem {
    pub service: String,
    pub widget: Option<gtk4::Image>,
    popover_menu: Option<gtk4::PopoverMenu>
}

impl SystemTrayItem {
    pub fn new(service: String) -> Self {
        Self {
            service,
            ..Self::default()
        }
    }

    pub fn build_widget(&mut self) {
        let empty_model: gio::MenuModel = gio::Menu::new().into();
        let popover_menu = gtk4::PopoverMenu::from_model(Some(&empty_model));

        popover_menu.set_css_classes(&["bar-tray-popover-menu"]);

        self.popover_menu = Some(popover_menu);

        let default_activate = gesture::on_primary_down({
            let service = self.service.clone();
        
            move |_, x, y| {
                if let Some(item) = tray::try_read_item(&service) {
                    let _ = item.activate(x as i32, y as i32);
                }
            }
        });

        let menus_activate = gesture::on_secondary_down({
            let service = self.service.clone();
            let popover_menu = self.popover_menu.clone();

            move |_, _, _| if let Some(popover_menu) = &popover_menu {
                let (sender, receiver) = async_channel::bounded::<(StatusNotifierItem, Menu)>(1);

                tokio::spawn({
                    let service = service.clone();
                    async move {
                        if let Some(item) = tray::try_read_item(&service) {
                            if let Ok(menu_layout) = item.menu.get_layout() {
                                let _ = sender.send((item, menu_layout)).await;
                            }
                        }
                    }
                });

                gtk4::glib::spawn_future_local({
                    let popover_menu = popover_menu.clone();
                    async move {
                        if let Ok((item, layout)) = receiver.recv().await {
                            if let Some((model, actions)) = tray_menu::build_gio_dbus_menu_model_with_layout(item, &layout) {
                                popover_menu.set_menu_model(Some(&model));
                                popover_menu.insert_action_group("dbusmenu", Some(&actions));
                            }

                            popover_menu.popup();
                        }
                    }
                });
            }
        });

        view! {
            new_widget = gtk4::Image {
                set_css_classes: &["bar-tray-item"],
                set_pixel_size: 14,
                add_controller: default_activate,
                add_controller: menus_activate
            }
        };

        if let Some(item) = crate::singletons::tray::try_read_item(&self.service) {
            if !item.icon_pixmap.is_empty() {
                new_widget.set_from_pixbuf(make_icon_pixbuf(Some(&item.icon_pixmap)).as_ref());
            } else {
                let icon_pixbuf = get_pixbuf_or_fallback(&item.icon_name, "emote-love");
                new_widget.set_from_pixbuf(icon_pixbuf.as_ref());
            }
            
            if !item.tool_tip.title.is_empty() {
                new_widget.set_tooltip_text(Some(&item.tool_tip.title));
            }
        }

        // Set the widget
        self.popover_menu.as_ref().unwrap().set_parent(&new_widget);
        self.widget = Some(new_widget);
    }

    pub fn update(&self, member: &str) {
        if let (Some(widget), Some(item)) = (&self.widget, crate::singletons::tray::try_read_item(&self.service)) {
            match member {
                "NewToolTip" => {
                    if !item.tool_tip.title.is_empty() {
                        widget.set_tooltip_text(Some(&item.tool_tip.title));
                    }
                },

                "NewIcon" => {
                    if !item.icon_pixmap.is_empty() {
                        widget.set_from_pixbuf(make_icon_pixbuf(Some(&item.icon_pixmap)).as_ref());
                    } else {
                        let icon_pixbuf = get_pixbuf_or_fallback(&item.icon_name, "emote-love");
                        widget.set_from_pixbuf(icon_pixbuf.as_ref());
                    }
                },

                _ => {}
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
        box_.set_visible(false);
        box_.set_hexpand(false);

        let items = Vec::new();

        Self {
            box_,
            items,
        }
    }

    fn update_visibility(&self) {
        if self.items.is_empty() {
            self.box_.hide();
        } else {
            self.box_.show();
        }
    }

    fn add_item(&mut self, service: String) {
        let item = SystemTrayItem::new(service);

        self.items.push(item);
        self.update_visibility();
    }

    fn build_item(&mut self, service: &str) {
        if let Some(item) = self.items.iter_mut().find(|i| i.service == service) {
            item.build_widget();

            if let Some(widget) = &item.widget {
                self.box_.append(widget);
            }
        }
    }

    fn update_item(&mut self, member: &str, service: &str) {
        if let Some(item) = self.items.iter_mut().find(|i| i.service == service) {
            item.update(member);
        }
    }

    fn remove_item(&mut self, service: &str) {
        if let Some(pos) = self.items.iter().position(|i| i.service == service) {
            let item = self.items.remove(pos);

            if let Some(popover_menu) = item.popover_menu {
                popover_menu.unparent();
            }

            if let Some(widget) = item.widget {
                self.box_.remove(&widget);
            }

            self.update_visibility();
        }
    }

    fn get_widget(&self) -> gtk4::Box {
        self.box_.clone()
    }
}

pub fn new() -> gtk4::Box {
    let mut tray = SystemTray::new();
    let widget = tray.get_widget();

    // We may have missed some items that were registered before we start listening.
    // Fetch the current items and register them.
    if let Some(items) = crate::singletons::tray::ITEMS.get() {
        for item in &items.try_read().map_or(Vec::new(), |items| items.clone()) {
            tray.add_item(item.service.clone());
        }

        tray.items.iter_mut().for_each(|item| {
            item.build_widget();
            
            if let Some(widget) = &item.widget {
                tray.box_.append(widget);
            }
        });
    } else {
        eprintln!("Failed to fetch current tray items.");
    }

    // Watch for tray events
    let (tx, rx) = async_channel::bounded::<BusEvent>(1);
    tokio::spawn(async move {
        while let Ok(event) = subscribe().recv().await {
            tx.send(event).await.unwrap();
        }
    });

    gtk4::glib::spawn_future_local(async move {
        while let Ok(event) = rx.recv().await {
            match event {
                BusEvent::ItemRegistered(item) => {
                    tray.add_item(item.service.clone());
                    tray.build_item(&item.service);
                },

                BusEvent::ItemUpdated(member, item) => {
                    tray.update_item(&member, &item.service);
                },

                BusEvent::ItemUnregistered(item) => {
                    tray.remove_item(&item.service);
                }
            }
        }
    });

    SimpleBarModuleWrapper::new(&widget).get_widget()
}