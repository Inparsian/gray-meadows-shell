mod entry;

use std::{cell::RefCell, rc::Rc};
use gtk4::prelude::*;
use relm4::RelmRemoveAllExt as _;

use crate::ipc;
use crate::singletons::clipboard;
use self::entry::clipboard_entry;
use super::fullscreen::FullscreenWindow;

pub fn new(application: &libadwaita::Application) -> FullscreenWindow {
    let entries: Rc<RefCell<Vec<(usize, String)>>> = Rc::new(RefCell::new(clipboard::fetch_clipboard_entries()));

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
            connect_changed: clone!(
                #[weak] listbox,
                #[strong] entries,
                move |entry| {
                    let text = entry.text().to_string();
                    listbox.remove_all();
                    for (id, preview) in entries.borrow().iter() {
                        if preview.to_lowercase().contains(&text.to_lowercase()) {
                            listbox.append(&clipboard_entry(*id, preview));
                        }
                    }
                }
            )
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

    ipc::listen_for_messages_local(clone!(
        #[weak] listbox,
        #[strong] entries,
        move |message| {
            if message.as_str() == "update_clipboard_window_entries" {
                // Tell the window to update its entries
                let new_entries = clipboard::fetch_clipboard_entries();
                *entries.borrow_mut() = new_entries;
                listbox.remove_all();
                for (id, preview) in entries.borrow().iter() {
                    listbox.append(&clipboard_entry(*id, preview));
                }
            }
        }
    ));

    for (id, preview) in entries.borrow().iter() {
        listbox.append(&clipboard_entry(*id, preview));
    }

    let fullscreen = FullscreenWindow::new(
        application,
        &["clipboard-window"],
        &child,
    );

    fullscreen.window.connect_unmap(clone!(
        #[weak] entry,
        move |_| {
            entry.set_text("");
        }
    ));

    fullscreen.window.connect_map(move |_| {
        entry.grab_focus();
        scrollable.vadjustment().set_value(0.0);
    });

    fullscreen
}