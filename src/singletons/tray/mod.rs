pub mod tray_icon;
pub mod tray_menu;

use futures_signals::signal_vec::MutableVec;
use once_cell::sync::{Lazy, OnceCell};
use system_tray::{client::{Client, Event, UpdateEvent}, item::StatusNotifierItem, menu::TrayMenu};

pub static TRAY_CLIENT: OnceCell<Client> = OnceCell::new();
pub static TRAY_ITEMS: Lazy<MutableVec<(String, StatusNotifierItem)>> = Lazy::new(MutableVec::new);

pub fn get_tray_item(owner: &str) -> Option<StatusNotifierItem> {
    TRAY_ITEMS.lock_ref().iter().find(|(o, _)| o == owner).map(|(_, item)| item.clone())
}

pub fn get_tray_menu(owner: &str) -> Option<TrayMenu> {
    TRAY_CLIENT.get().unwrap().items().lock().unwrap().iter()
        .find(|(o, _)| **o == owner)
        .and_then(|(_, item)| item.1.as_ref())
        .cloned()
}

pub fn activate() {
    tokio::spawn(async move {
        let client = Client::new().await.unwrap();
        let mut tray_rx = client.subscribe();
        let initial_items = client.items();
        
        println!("Initial tray items: {:?}", initial_items);

        let _ = TRAY_CLIENT.set(client);
        
        while let Ok(event) = tray_rx.recv().await {
            match event {
                Event::Add(owner, item) => {
                    println!("Tray item added: {:?}", owner);
                    let mut item = *item;
                    tray_icon::compress_icon(&mut item);
                    TRAY_ITEMS.lock_mut().push_cloned((owner, item));
                },

                Event::Update(owner, update_event) => {
                    let mut items_mut = TRAY_ITEMS.lock_mut();
                    let existing_index = items_mut.iter().position(|i| i.0 == owner)
                        .unwrap_or(usize::MAX); // Default to an impossible index if not found

                    if let Some(existing) = items_mut.get(existing_index) {
                        let mut item = existing.1.clone();

                        match update_event {
                            UpdateEvent::AttentionIcon(icon) => {
                                println!("Updating attention icon for item: {:?}", owner);
                                item.attention_icon_name = icon;
                            },

                            UpdateEvent::OverlayIcon(icon) => {
                                println!("Updating overlay icon for item: {:?}", owner);
                                item.overlay_icon_name = icon;
                            },

                            UpdateEvent::Icon { icon_name, icon_pixmap } => {
                                println!("Updating icon for item: {:?}", owner);
                                item.icon_name = icon_name;
                                item.icon_pixmap = tray_icon::compress_icon_pixmap(&icon_pixmap);
                            },

                            UpdateEvent::Tooltip(tooltip) => {
                                println!("Updating tooltip for item: {:?}", owner);
                                item.tool_tip = tooltip;
                            },

                            UpdateEvent::Status(status) => {
                                println!("Updating status for item: {:?} to {:?}", owner, status);
                                item.status = status;
                            },

                            UpdateEvent::Title(title) => {
                                println!("Updating title for item: {:?}", owner);
                                item.title = title;
                            },

                            // TODO: Handle tray item menus
                            UpdateEvent::Menu(_) => {
                                println!("Updating menu for item: {:?}", owner);
                            },

                            UpdateEvent::MenuConnect(_) => {
                                println!("New menu connected to item: {:?}", owner);
                            },

                            UpdateEvent::MenuDiff(_) => {
                                println!("Menu props have changed for item: {:?}", owner);
                            }
                        }

                        items_mut.set_cloned(existing_index, (owner, item));
                    }
                },

                Event::Remove(owner) => {
                    println!("Tray item removed: {:?}", owner);
                    TRAY_ITEMS.lock_mut().retain(|i| i.0 != owner);
                }
            }
        }
    });
}