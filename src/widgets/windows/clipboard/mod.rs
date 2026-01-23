mod data;
mod entry;

use gtk4::prelude::*;
use std::collections::HashSet;

use crate::ipc;
use crate::singletons::clipboard;
use self::data::ClipboardEntryData;
use self::entry::ClipboardEntry;
use super::fullscreen::FullscreenWindow;

fn apply_filter_to_model(model: &gio::ListStore, query: &str) {
    let query = query.trim().to_lowercase();
    let cache = clipboard::get_all_previews();
    let mut desired_ids: Vec<i32> = cache
        .iter()
        .filter(|(_, preview)| query.is_empty() || preview.to_lowercase().contains(&query))
        .map(|(id, _)| *id)
        .collect();

    desired_ids.sort_by_key(|id| std::cmp::Reverse(*id));
    let desired_set: HashSet<i32> = desired_ids.iter().copied().collect();

    for idx in (0..model.n_items()).rev() {
        let obj = model
            .item(idx)
            .and_downcast::<ClipboardEntryData>()
            .expect("ListStore contained non-ClipboardEntryData item");

        if !desired_set.contains(&obj.id()) {
            model.remove(idx);
        }
    }

    for (i, desired_id) in desired_ids.iter().copied().enumerate() {
        let i = i as u32;
        if i >= model.n_items() {
            model.append(&ClipboardEntryData::new(desired_id));
            continue;
        }

        let current = model
            .item(i)
            .and_downcast::<ClipboardEntryData>()
            .expect("ListStore contained non-ClipboardEntryData item");

        if current.id() != desired_id {
            let found_at = ((i + 1)..model.n_items()).find(|&j| {
                let obj = model
                    .item(j)
                    .and_downcast::<ClipboardEntryData>()
                    .expect("ListStore contained non-ClipboardEntryData item");

                obj.id() == desired_id
            });

            if let Some(from_index) = found_at {
                let obj = model
                    .item(from_index)
                    .and_downcast::<ClipboardEntryData>()
                    .expect("ListStore contained non-ClipboardEntryData item");

                model.remove(from_index);
                model.insert(i, &obj);
            } else {
                model.insert(i, &ClipboardEntryData::new(desired_id));
            }
        }
    }

    while model.n_items() > desired_ids.len() as u32 {
        model.remove(model.n_items() - 1);
    }
}

pub fn new(application: &libadwaita::Application) -> FullscreenWindow {
    let model = gio::ListStore::new::<ClipboardEntryData>();
    apply_filter_to_model(&model, "");

    let factory = gtk4::SignalListItemFactory::new();
    factory.connect_setup(move |_, item| {
        let clipboard_entry = ClipboardEntry::default();
        let list_item = item.downcast_ref::<gtk4::ListItem>()
            .expect("Expected a ListItem");

        list_item.set_child(Some(&clipboard_entry));
    });
    factory.connect_bind(move |_, item| {
        let list_item = item
            .downcast_ref::<gtk4::ListItem>()
            .expect("Expected a ListItem");

        let entry_data = list_item
            .item()
            .and_downcast::<ClipboardEntryData>()
            .expect("Expected a ClipboardEntryData");

        let clipboard_entry = list_item
            .child()
            .and_downcast::<ClipboardEntry>()
            .expect("Expected a ClipboardEntry");

        let id = entry_data.id();
        clipboard_entry.set_id(id);
        clipboard_entry.refresh();
    });

    let selection_model = gtk4::SingleSelection::new(Some(model.clone().upcast::<gio::ListModel>()));
    let list_view = gtk4::ListView::builder()
        .model(&selection_model)
        .factory(&factory)
        .hexpand(true)
        .vexpand(true)
        .css_classes(["clipboard-listbox"])
        .build();

    view! {
        scrollable = gtk4::ScrolledWindow {
            set_vexpand: true,
            set_hexpand: true,
            set_min_content_width: 400,
            set_min_content_height: 450,
            set_child: Some(&list_view),
        },

        entry = gtk4::Entry {
            set_css_classes: &["filter-entry-prompt"],
            set_placeholder_text: Some("Filter clipboard entries..."),
            set_hexpand: true,
            set_has_frame: false,
            connect_changed: clone!(
                #[weak] model,
                move |entry| apply_filter_to_model(&model, &entry.text())
            ),
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
        #[weak] model,
        #[weak] entry,
        #[weak] scrollable,
        move |message| {
            if message.as_str() == "update_clipboard_window_entries" {
                clipboard::refresh_clipboard_entries();
                apply_filter_to_model(&model, &entry.text());
                
                glib::idle_add_local_once(clone!(
                    #[weak] scrollable,
                    move || {
                        scrollable.vadjustment().set_value(0.0);
                    }
                ));
            }
        }
    ));

    let fullscreen = FullscreenWindow::new(
        application,
        &["clipboard-window"],
        &child,
    );

    fullscreen.window.connect_unmap(clone!(
        #[weak] entry,
        #[weak] scrollable,
        move |_| {
            entry.set_text("");
            scrollable.vadjustment().set_value(0.0);
        }
    ));

    fullscreen.window.connect_map(move |_| {
        entry.grab_focus();
        scrollable.vadjustment().set_value(0.0);
    });

    fullscreen
}