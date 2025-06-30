use futures_signals::signal_vec::MutableVec;
use gdk4::glib::Bytes;
use once_cell::sync::Lazy;
use system_tray::{client::{Client, Event, UpdateEvent}, item::StatusNotifierItem};

// Rationale: Some icons have the possibility of being absurdly large (e.g. 1024x1024). This poses
// several issues - large icons consume a ton of memory, take literally forever to render, and take
// lots of processing power to render as well. Nothing much I can do about the memory consumption,
// but I can at least compress the icon data before rendering it to a pixbuf.
const C_WIDTH: u32 = 32;
const C_HEIGHT: u32 = 32;

pub static TRAY_ITEMS: Lazy<MutableVec<(String, StatusNotifierItem)>> = Lazy::new(MutableVec::new);

pub fn get_tray_item(owner: &str) -> Option<StatusNotifierItem> {
    TRAY_ITEMS.lock_ref().iter().find(|(o, _)| o == owner).map(|(_, item)| item.clone())
}

pub fn make_icon_pixbuf(item: StatusNotifierItem) -> gtk4::gdk_pixbuf::Pixbuf {
    let argb32_icon = item.icon_pixmap.clone().unwrap_or_default();

    // Pick the icon that is closest to C_WIDTHxC_HEIGHT.
    let closest_icon = argb32_icon.iter()
        .min_by_key(|pixmap| {
            let width = pixmap.width;
            let height = pixmap.height;

            (width - C_WIDTH as i32).abs() + (height - C_HEIGHT as i32).abs()
        });

    if let Some(icon) = closest_icon {
        // Perform pixel compression if icon.width and icon.height are larger than C_WIDTH and C_HEIGHT
        let should_compress = icon.width > C_WIDTH as i32 || icon.height > C_HEIGHT as i32;
        let compressed_pixels = if should_compress {
            let mut vec = Vec::new();

            for y in 0..C_HEIGHT {
                for x in 0..C_WIDTH {
                    let c_icon_x = (x as f32 / C_WIDTH as f32 * icon.width as f32) as u32;
                    let c_icon_y = (y as f32 / C_HEIGHT as f32 * icon.height as f32) as u32;
                    let c_icon_index = (c_icon_y * icon.width as u32 + c_icon_x) as usize * 4;

                    if c_icon_index < icon.pixels.len() {
                        // push the next 4 items (a, r, g, b)
                        for c in 0..4 {
                            let pixel_index = c_icon_index + c;
                            if pixel_index < icon.pixels.len() {
                                vec.push(icon.pixels[pixel_index]);
                            }
                        }
                    }
                }
            }

            vec
        } else {
            icon.pixels.clone() // leave as is
        };

        if should_compress {
            println!("Compressing: {}x{}", icon.width, icon.height);
        } else {
            println!("Using original icon size: {}x{}", icon.width, icon.height);
        }

        let pixbuf = gtk4::gdk_pixbuf::Pixbuf::from_mut_slice(
            compressed_pixels,
            gtk4::gdk_pixbuf::Colorspace::Rgb,
            true,
            8,
            if should_compress { C_WIDTH as i32 } else { icon.width },
            if should_compress { C_HEIGHT as i32 } else { icon.height },
            if should_compress { C_WIDTH as i32 * 4 } else { icon.width * 4 }
        );

        // aesthetic thing
        pixbuf.saturate_and_pixelate(
            &pixbuf,
            0.0,
            false
        );

        pixbuf
    } else {
        gtk4::gdk_pixbuf::Pixbuf::from_bytes(
            &Bytes::from(&[]),
            gtk4::gdk_pixbuf::Colorspace::Rgb,
            true,
            8,
            0,
            0,
            0
        )
    }
}

pub fn activate() {
    tokio::spawn(async move {
        let client = Client::new().await.unwrap();
        let mut tray_rx = client.subscribe();

        let initial_items = client.items();
        
        println!("Initial tray items: {:?}", initial_items);
        
        while let Ok(event) = tray_rx.recv().await {
            match event {
                Event::Add(owner, item) => {
                    println!("Tray item added: {:?}", owner);
                    TRAY_ITEMS.lock_mut().push_cloned((owner, *item));
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
                                item.icon_pixmap = icon_pixmap;
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