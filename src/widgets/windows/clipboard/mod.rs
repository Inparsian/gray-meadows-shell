use std::{cell::RefCell, collections::HashMap, io::{Read, Write}, process::Stdio, rc::Rc, sync::{LazyLock, Mutex}};
use gtk4::prelude::*;
use regex::Regex;
use relm4::RelmRemoveAllExt;
use image::codecs::png::PngEncoder;
use image::imageops::FilterType;
use image::ImageEncoder;

use crate::{color, ipc, widgets::windows::{self, fullscreen::FullscreenWindow}};

static IMAGE_SIZE: u32 = 192;
static IMAGE_BINARY_DATA_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[\[ binary data (\d+) ([KMGT]i)?B (\w+) (\d+)x(\d+) \]\]").expect("Failed to compile image binary data regex")
});
static DECODE_CACHE: LazyLock<Mutex<HashMap<usize, Vec<u8>>>> = LazyLock::new(|| {
    Mutex::new(HashMap::new())
});

pub fn decode_clipboard_entry(id: &str) -> Option<Vec<u8>> {
    if DECODE_CACHE.lock().unwrap().contains_key(&id.parse::<usize>().ok()?) {
        return DECODE_CACHE.lock().unwrap().get(&id.parse::<usize>().ok()?).cloned();
    }

    // fetch the image data from cliphist
    let output = std::process::Command::new("cliphist")
        .arg("decode")
        .arg(id)
        .output()
        .ok()?;

    output.status.success().then_some(output.stdout)
        .inspect(|data| if let Ok(parsed_id) = id.parse::<usize>() {
            DECODE_CACHE.lock().unwrap().insert(parsed_id, data.clone());
        })
}

pub fn is_an_image_clipboard_entry(preview: &str) -> bool {
    IMAGE_BINARY_DATA_PATTERN.is_match(preview)
}

pub fn fetch_clipboard_entries() -> Vec<(usize, String)> {
    if let Ok(output) = std::process::Command::new("cliphist")
        .arg("list")
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout.lines().filter_map(|line| {
                let mut parts = line.splitn(2, '\t');
                let id_str = parts.next()?.trim();
                let preview = parts.next()?.trim().to_owned();
                let id = id_str.parse::<usize>().ok()?;
                Some((id, preview))
            }).collect();
        }
    }

    Vec::new()
}

pub fn copy_entry(id: usize) {
    // pipe cliphist decode <id> to wl-copy
    std::thread::spawn(move || {
        let decode_process = std::process::Command::new("cliphist")
            .arg("decode")
            .arg(id.to_string())
            .stdout(Stdio::piped())
            .spawn();

        if let Ok(mut decode_child) = decode_process {
            if let Some(mut decode_stdout) = decode_child.stdout.take() {
                let mut buffer = Vec::new();

                if let Ok(bytes_read) = decode_stdout.read_to_end(&mut buffer) {
                    if bytes_read == 0 {
                        return;
                    }

                    // if the buffer converted to lossy utf8 is one character, copy it with a output call,
                    // wl-copy doesn't like single character inputs to it's stdin for some reason
                    let lossy_utf8 = String::from_utf8_lossy(&buffer);
                    if lossy_utf8.chars().count() == 1 {
                        let _ = std::process::Command::new("wl-copy")
                            .arg(lossy_utf8.to_string())
                            .output();
                        return;
                    }

                    // otherwise the buffer can be piped as usual
                    let wl_copy_process = std::process::Command::new("wl-copy")
                        .stdin(Stdio::piped())
                        .spawn();

                    if let Ok(mut wl_copy_child) = wl_copy_process {
                        if let Some(mut wl_copy_stdin) = wl_copy_child.stdin.take() {
                            let _ = wl_copy_stdin.write_all(&buffer);
                        }
                        let _ = wl_copy_child.wait();
                    }
                }
            }
            let _ = decode_child.wait();
        }
    });
}

fn clipboard_entry(id: usize, preview: &str) -> gtk4::Button {
    let button = gtk4::Button::new();
    button.set_css_classes(&["clipboard-entry"]);

    let child: gtk4::Widget = if is_an_image_clipboard_entry(preview) {
        let picture = gtk4::Picture::new();
        picture.set_halign(gtk4::Align::Start);
        picture.set_valign(gtk4::Align::Center);
        picture.set_keep_aspect_ratio(true);

        let (tx, rx) = async_channel::unbounded::<(u32, u32, Vec<u8>)>();
        tokio::spawn(async move {
            if let Some(decoded) = decode_clipboard_entry(&id.to_string()) {
                let image = image::load_from_memory(&decoded).ok();
                if let Some(img) = image {
                    let scaled_img = if img.width() > IMAGE_SIZE || img.height() > IMAGE_SIZE {
                        let aspect = img.width() as f32 / img.height() as f32;
                        let new_width = if aspect >= 1.0 {
                            IMAGE_SIZE
                        } else {
                            (IMAGE_SIZE as f32 * aspect) as u32
                        };
                        let new_height = if aspect >= 1.0 {
                            (IMAGE_SIZE as f32 / aspect) as u32
                        } else {
                            IMAGE_SIZE
                        };
                        img.resize(new_width, new_height, FilterType::Lanczos3)
                    } else {
                        img
                    };

                    let mut buf = Vec::new();
                    if PngEncoder::new(&mut buf).write_image(
                        scaled_img.as_bytes(),
                        scaled_img.width(),
                        scaled_img.height(),
                        scaled_img.color().into()
                    ).is_ok() {
                        let _ = tx.send((scaled_img.width(), scaled_img.height(), buf)).await;
                    }
                }
            }
        });

        gtk4::glib::spawn_future_local({
            let picture = picture.clone();
            async move {
                if let Ok((width, height, decoded)) = rx.recv().await {
                    let loader = gtk4::gdk_pixbuf::PixbufLoader::new();
                    if loader.write(&decoded).is_ok() {
                        let _ = loader.close();
                        if let Some(pixbuf) = loader.pixbuf() {
                            picture.set_pixbuf(Some(&pixbuf));
                            picture.set_width_request(width as i32);
                            picture.set_height_request(height as i32);
                        }
                    }
                }
            }
        });

        picture.upcast()
    } else if let Some(hex) = color::parse_color_into_hex(preview) {
        let box_ = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        
        let color_style_provider = gtk4::CssProvider::new();
        let color_style = format!(".color-preview-box {{ background-color: {}; }}", hex);
        let color_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        color_box.set_css_classes(&["color-preview-box"]);
        color_box.set_valign(gtk4::Align::Center);
        color_box.style_context().add_provider(&color_style_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
        color_style_provider.load_from_data(&color_style);

        let label = gtk4::Label::new(Some(preview));
        label.set_hexpand(true);
        label.set_xalign(0.0);
        label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        box_.append(&color_box);
        box_.append(&label);
        box_.upcast()
    } else {
        let label = gtk4::Label::new(Some(preview));
        label.set_hexpand(true);
        label.set_xalign(0.0);
        label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        label.upcast()
    };

    button.connect_clicked(move |_| {
        copy_entry(id);
        windows::hide("clipboard");
    });

    // stupid layout trick that helps the button not vertically stretch more than needed
    let bx = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    let bxend = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    bxend.set_halign(gtk4::Align::End);
    bx.append(&child);
    bx.append(&bxend);
    button.set_child(Some(&bx));

    button
}

pub fn new(application: &libadwaita::Application) -> FullscreenWindow {
    let entries: Rc<RefCell<Vec<(usize, String)>>> = Rc::new(RefCell::new(fetch_clipboard_entries()));

    view! {
        listbox = gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 0,
            set_hexpand: true,
            set_vexpand: true,
            set_css_classes: &["clipboard-listbox"],
        },

        scrollable = gtk4::ScrolledWindow {
            set_vexpand: true,
            set_hexpand: true,
            set_min_content_width: 400,
            set_min_content_height: 450,
            set_child: Some(&listbox),
        },

        entry = gtk4::Entry {
            set_css_classes: &["filter-entry-prompt"],
            set_placeholder_text: Some("Filter clipboard entries..."),
            set_hexpand: true,
            set_has_frame: false,
            connect_changed: {
                let listbox_clone = listbox.clone();
                let entries_clone = entries.clone();
                move |entry| {
                    let text = entry.text().to_string();
                    listbox_clone.remove_all();
                    for (id, preview) in entries_clone.borrow().iter() {
                        if preview.to_lowercase().contains(&text.to_lowercase()) {
                            listbox_clone.append(&clipboard_entry(*id, preview));
                        }
                    }
                }
            }
        },

        filter_entry_box = gtk4::Box {
            set_css_classes: &["filter-entry-box"],
            set_orientation: gtk4::Orientation::Horizontal,
            set_hexpand: true,

            gtk4::Label {
                set_css_classes: &["filter-entry-icon"],
                set_label: "search",
                set_halign: gtk4::Align::Start,
            },

            append: &entry,
        },

        child = gtk4::Box {
            set_css_classes: &["clipboard-window-content"],
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 0,
            set_halign: gtk4::Align::Center,
            set_valign: gtk4::Align::Center,

            append: &filter_entry_box,
            append: &scrollable,
        }
    }

    ipc::listen_for_messages_local({
        let listbox = listbox.clone();
        let entries = entries.clone();
        move |message| {
            if message.as_str() == "update_clipboard_window_entries" {
                // Tell the window to update its entries
                let new_entries = fetch_clipboard_entries();
                *entries.borrow_mut() = new_entries;
                listbox.remove_all();
                for (id, preview) in entries.borrow().iter() {
                    listbox.append(&clipboard_entry(*id, preview));
                }
            }
        }
    });

    for (id, preview) in entries.borrow().iter() {
        listbox.append(&clipboard_entry(*id, preview));
    }

    let fullscreen = FullscreenWindow::new(
        application,
        &["clipboard-window"],
        &child,
    );

    fullscreen.window.connect_unmap({
        let entry = entry.clone();
        move |_| {
            entry.set_text("");
        }
    });

    fullscreen.window.connect_map({
        move |_| {
            entry.grab_focus();
            scrollable.vadjustment().set_value(0.0);
        }
    });

    fullscreen
}