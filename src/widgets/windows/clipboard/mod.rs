use std::{cell::RefCell, process::Stdio, rc::Rc, collections::HashMap, sync::{LazyLock, Mutex}};
use gtk4::prelude::*;
use regex::Regex;
use relm4::RelmRemoveAllExt;
use image::codecs::png::PngEncoder;
use image::ImageEncoder;

use crate::{color, ipc, widgets::windows::{self, fullscreen::FullscreenWindow}};

static IMAGE_WIDTH: u32 = 192;
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
            if let Some(decode_stdout) = decode_child.stdout.take() {
                let wl_copy_process = std::process::Command::new("wl-copy")
                    .stdin(Stdio::from(decode_stdout))
                    .spawn();

                if let Ok(mut wl_copy_child) = wl_copy_process {
                    let _ = wl_copy_child.wait();
                }
            }
            let _ = decode_child.wait();
        }
    });
}

fn clipboard_entry(id: usize, preview: &str) -> gtk4::Button {
    let button = gtk4::Button::new();
    button.set_css_classes(&["clipboard-entry"]);

    if is_an_image_clipboard_entry(preview) {
        let image = gtk4::Image::new();
        image.set_halign(gtk4::Align::Start);
        image.set_valign(gtk4::Align::Center);
        image.set_pixel_size(IMAGE_WIDTH as i32);
        button.set_child(Some(&image));

        let (tx, rx) = async_channel::unbounded::<Vec<u8>>();
        tokio::spawn(async move {
            if let Some(decoded) = decode_clipboard_entry(&id.to_string()) {
                let image = image::load_from_memory(&decoded).ok();
                if let Some(img) = image {
                    let scaled_img = if img.width() > IMAGE_WIDTH {
                        img.resize(
                            IMAGE_WIDTH, 
                            (img.height() as f32 * (IMAGE_WIDTH as f32 / img.width() as f32)) as u32, 
                            image::imageops::FilterType::Lanczos3
                        )
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
                        let _ = tx.send(buf).await;
                    }
                }
            }
        });

        gtk4::glib::spawn_future_local(async move {
            if let Ok(decoded) = rx.recv().await {
                let loader = gtk4::gdk_pixbuf::PixbufLoader::new();
                if loader.write(&decoded).is_ok() {
                    let _ = loader.close();
                    if let Some(pixbuf) = loader.pixbuf() {
                        image.set_from_pixbuf(Some(&pixbuf));
                    }
                }
            }
        });
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
        label.set_halign(gtk4::Align::Start);
        label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        box_.append(&color_box);
        box_.append(&label);
        button.set_child(Some(&box_));
    } else {
        let label = gtk4::Label::new(Some(preview));
        label.set_halign(gtk4::Align::Start);
        label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        button.set_child(Some(&label));
    }

    button.connect_clicked(move |_| {
        copy_entry(id);
        windows::hide("clipboard");
    });

    button
}

pub fn new(application: &libadwaita::Application) -> FullscreenWindow {
    let entries: Rc<RefCell<Vec<(usize, String)>>> = Rc::new(RefCell::new(fetch_clipboard_entries()));

    view! {
        listbox = gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 0,
            set_halign: gtk4::Align::Center,
            set_valign: gtk4::Align::Center,
            set_css_classes: &["clipboard-listbox"],
        },

        scrollable = gtk4::ScrolledWindow {
            set_vexpand: true,
            set_hexpand: true,
            set_min_content_width: 600,
            set_min_content_height: 450,
            set_child: Some(&listbox),
        },

        child = gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 0,
            set_halign: gtk4::Align::Center,
            set_valign: gtk4::Align::Center,

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

    FullscreenWindow::new(
        application,
        &["clipboard-window"],
        &child,
    )
}