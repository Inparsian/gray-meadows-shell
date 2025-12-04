use std::{cell::RefCell, process::Stdio, rc::Rc, sync::LazyLock};
use gtk4::prelude::*;
use regex::Regex;
use relm4::RelmRemoveAllExt;

use crate::{ipc, widgets::windows::{self, fullscreen::FullscreenWindow}};

static IMAGE_BINARY_DATA_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[\[ binary data (\d+) (KiB|MiB) (\w+) (\d+)x(\d+) \]\]").expect("Failed to compile image binary data regex")
});

pub fn decode_clipboard_entry(id: &str) -> Option<Vec<u8>> {
    // fetch the image data from cliphist
    let output = std::process::Command::new("cliphist")
        .arg("decode")
        .arg(id)
        .output()
        .ok()?;

    output.status.success().then_some(output.stdout)
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
        let decoded = decode_clipboard_entry(&id.to_string());
        if let Some(image_data) = decoded {
            let loader = gtk4::gdk_pixbuf::PixbufLoader::new();
            if loader.write(&image_data).is_ok() {
                let _ = loader.close();
                if let Some(pixbuf) = loader.pixbuf() {
                    let image = gtk4::Image::from_pixbuf(Some(&pixbuf));
                    image.set_halign(gtk4::Align::Start);
                    image.set_pixel_size(256);
                    button.set_child(Some(&image));
                }
            }
        }
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
            set_min_content_width: 400,
            set_min_content_height: 300,
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