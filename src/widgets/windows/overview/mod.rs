mod item;
mod list;
mod modules;
mod windows;

use std::{cell::RefCell, rc::Rc, sync::LazyLock};
use freedesktop_desktop_entry::get_languages_from_env;
use gtk4::prelude::*;
use regex::Regex;
use urlencoding::encode;

use crate::ipc;
use crate::singletons::apps;
use crate::utils::gesture;
use self::item::{OverviewSearchItem, OverviewSearchItemAction};
use self::list::{OverviewSearchList, get_button_from_row};
use self::modules::{OverviewSearchModule, input_without_extensions, validate_input};
use self::windows::{frequent::OverviewFrequentWindow, recent::OverviewRecentWindow};
use super::fullscreen::FullscreenWindow;

static MODULES: LazyLock<Vec<&(dyn OverviewSearchModule + Send + Sync)>> = LazyLock::new(|| vec![
    &modules::calculator::OverviewCalculatorModule,
    &modules::text::OverviewTextModule,
    &modules::terminal::OverviewTerminalModule,
    &modules::hashing::OverviewHashingModule
]);

static ALPHANUMERIC_SYMBOLIC_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new("^[a-zA-Z0-9 ~!@#$%^&*()_+\\-=\\[\\]{}|;':\",./<>?]+$").expect("Failed to compile alphanumeric symbolic regex")
});

fn generate_entry_box_icon_stack() -> gtk4::Stack {
    let stack = gtk4::Stack::new();
    stack.set_css_classes(&["entry-box-icon"]);
    stack.set_transition_type(gtk4::StackTransitionType::SlideDown);
    stack.set_transition_duration(500);

    let default_label = gtk4::Label::new(Some("search"));
    stack.add_titled(&default_label, Some("search"), "search");
    stack.set_visible_child_name("search");

    for module in MODULES.iter() {
        let label = gtk4::Label::new(Some(module.icon()));
        stack.add_titled(&label, Some(module.icon()), module.icon());
    }

    stack
}

fn generate_search_results(query: &str) -> Vec<OverviewSearchItem> {
    let mut results = Vec::new();

    if MODULES.iter().any(|module| validate_input(*module, query)) {
        // Iterate through all modules and collect results
        for module in MODULES.iter() {
            if validate_input(*module, query) {
                let input = input_without_extensions(*module, query);
                for item in module.run(&input) {
                    results.push(item);
                }
            }
        }
    } else {
        // Filter and weigh the applications based on the query
        let locales = get_languages_from_env();
        let desktops = apps::query_desktops(query);
        for i in 0..8 {
            if let Some(entry) = desktops.get(i) {
                let entry = &entry.entry;

                results.push(OverviewSearchItem::new(
                    "application-result".to_owned(),
                    entry.name(&locales).unwrap_or_default().to_string(),
                    None,
                    entry.icon().map(|icon| icon.to_owned()).unwrap_or_default(),
                    "launch".to_owned(),
                    OverviewSearchItemAction::Launch(entry.exec().unwrap_or_default().to_owned()),
                    Some(query.to_owned())
                ));
            }
        }
    }

    // web search as final fallback
    results.push(OverviewSearchItem::new(
        "web-search".to_owned(),
        query.to_owned(),
        Some("Search the web".to_owned()),
        "search".to_owned(),
        "search".to_owned(),
        OverviewSearchItemAction::RunCommand(format!("xdg-open https://duckduckgo.com/?q={}", encode(query))),
        None
    ));

    results
}

pub fn new(application: &libadwaita::Application) -> FullscreenWindow {
    let search_results = Rc::new(RefCell::new(OverviewSearchList::new()));
    let frequent_window = OverviewFrequentWindow::new();
    let recent_window = OverviewRecentWindow::new();

    view! {
        entry_prompt_revealer = gtk4::Revealer {
            set_transition_type: gtk4::RevealerTransitionType::Crossfade,
            set_reveal_child: true,

            gtk4::Label {
                set_css_classes: &["entry-prompt-label"],
                set_label: "Type to search, and stuff",
            }
        },

        search_results_revealer = gtk4::Revealer {
            set_css_classes: &["overview-search-results-revealer"],
            set_transition_type: gtk4::RevealerTransitionType::SlideDown,
            set_transition_duration: 250,
            set_reveal_child: false,

            libadwaita::Clamp {
                set_width_request: 0,
                set_maximum_size: 0,
                set_child: Some(&search_results.borrow().get_widget())
            },
        },

        windows_box = gtk4::Box {
            set_css_classes: &["overview-windows-box"],
            set_orientation: gtk4::Orientation::Horizontal,
            set_spacing: 8,
            set_hexpand: true,
            set_vexpand: true,

            append: &frequent_window.widget,
            append: &recent_window.widget
        },

        windows_revealer = gtk4::Revealer {
            set_css_classes: &["overview-windows-revealer", "revealed"],
            set_transition_type: gtk4::RevealerTransitionType::SlideDown,
            set_transition_duration: 250,
            set_reveal_child: true,
            set_child: Some(&windows_box)
        },

        entry_box_icon = generate_entry_box_icon_stack(),

        entry_box = gtk4::Box {
            set_css_classes: &["entry-box"],
            set_orientation: gtk4::Orientation::Horizontal,
            set_halign: gtk4::Align::Center,
            append: &entry_box_icon,
        },

        entry = gtk4::Entry {
            set_css_classes: &["entry-prompt"],
            set_hexpand: true,
            set_has_frame: false,

            connect_activate: {
                let search_results = search_results.clone();

                move |entry| if !entry.text().to_string().is_empty() {
                    search_results.borrow().get_widget().first_child().map(|child| child.activate());
                }
            },

            connect_changed: {
                let entry_box = entry_box.clone();
                let entry_prompt_revealer = entry_prompt_revealer.clone();
                let search_results = search_results.clone();
                let search_results_revealer = search_results_revealer.clone();
                let windows_revealer = windows_revealer.clone();

                move |entry| {
                    if entry.text().is_empty() {
                        entry_prompt_revealer.set_reveal_child(true);
                        windows_revealer.add_css_class("revealed");
                        windows_revealer.set_reveal_child(true);
                        search_results_revealer.remove_css_class("revealed");
                        search_results_revealer.set_reveal_child(false);
                        entry_box.style_context().remove_class("entry-extended");
                        entry_box_icon.set_visible_child_name("search");
                    } else {
                        entry_prompt_revealer.set_reveal_child(false);
                        windows_revealer.remove_css_class("revealed");
                        windows_revealer.set_reveal_child(false);
                        search_results_revealer.add_css_class("revealed");
                        search_results_revealer.set_reveal_child(true);
                        entry_box.style_context().add_class("entry-extended");

                        let results = generate_search_results(&entry.text());

                        // Insert the new items into the search results list
                        let mut search_results_mut = search_results.borrow_mut();
                        for (i, item) in results.iter().enumerate() {
                            if let Some(index) = search_results_mut.items.iter().position(|r| r.smart_compare(item)) {
                                let existing_item = &mut search_results_mut.items[index];
                                if item.exact_id_comp_has() {
                                    if item.query.is_some() && existing_item.query != item.query {
                                        existing_item.query = item.query.clone();
                                        existing_item.set_title_markup();
                                    }
                                } else {
                                    existing_item.set_title_label(&item.title);

                                    if let Ok(action) = item.action.try_borrow() {
                                        existing_item.set_action(action.clone());
                                    }
                                }

                                // Move this item if its position has changed
                                if index != i {
                                    search_results_mut.move_item(index, i);
                                }
                            } else {
                                search_results_mut.insert(item, i);
                            }
                        }

                        // Remove items that are not in results
                        for (i, item) in search_results_mut.items.clone().iter().enumerate().rev() {
                            if !results.iter().any(|r| r.smart_compare(item)) {
                                search_results_mut.remove(i);
                            }
                        }

                        // Update the entry box icon if any module extensions matched
                        let mut matched_icon = "search";
                        for module in MODULES.iter() {
                            if validate_input(*module, &entry.text()) {
                                matched_icon = module.icon();
                                break;
                            }
                        }
                        entry_box_icon.set_visible_child_name(matched_icon);

                        search_results_mut.lock();
                    }
                }
            }
        },

        entry_overlay = gtk4::Overlay {
            set_hexpand: true,
            set_child: Some(&entry_prompt_revealer),
            add_overlay: &entry,
        },

        overview_box = gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 0,
            set_halign: gtk4::Align::Center,
            set_valign: gtk4::Align::Center,
            set_hexpand: true,

            append: &entry_box,
            append: &search_results_revealer,
            append: &windows_revealer
        }
    };

    entry_box.append(&entry_overlay);

    let fullscreen = FullscreenWindow::new(
        application,
        &["overview-window"],
        &overview_box,
    );

    fullscreen.window.connect_unmap({
        let entry = entry.clone();
        move |_| { entry.set_text(""); }
    });

    fullscreen.window.connect_map({
        let entry = entry.clone();
        move |_| { entry.grab_focus(); }
    });

    fullscreen.window.add_controller(gesture::on_key_press({
        let entry = entry.clone();
        let search_results = search_results.clone();

        // ListBoxRow steals the events for the Arrow Keys if it's focused, so
        // we can assume that it isn't focused if an event for Down is triggered
        // on the window
        move |val, _| if !entry.text().is_empty() && val.name() == Some("Down".into()) {
            let first_child = search_results.borrow().get_widget().first_child();

            first_child.map(|child| child.downcast_ref::<gtk4::ListBoxRow>().map(|row| {
                row.grab_focus();

                if let Some(button) = get_button_from_row(row) {
                    button.grab_focus();
                }
            }));
        }
    }));

    let search_results_widget = search_results.borrow().get_widget();
    search_results_widget.add_controller(gesture::on_key_press({
        let entry = entry.clone();
        let search_results_widget = search_results_widget.clone();

        move |val, _| {
            if val.name() == Some("Up".into()) {
                let first_child = search_results_widget.first_child();

                first_child.map(|child| child.downcast_ref::<gtk4::ListBoxRow>().map(|row| {
                    let button = get_button_from_row(row);

                    if button.is_some_and(|b| b.has_focus()) {
                        entry.grab_focus_without_selecting();
                    }
                }));
            }

            else if ["Left", "Right", "BackSpace", "Delete"].contains(&val.name().unwrap_or_default().as_str()) {
                entry.grab_focus_without_selecting();

                let text = entry.text().to_string();
                match val.name().as_deref() {
                    Some("Left" | "Right") => {
                        let pos = match val.name().as_deref() {
                            Some("Left") => text.len().saturating_sub(1),
                            Some("Right") => text.len(),
                            _ => 0
                        };
                
                        entry.select_region(pos as i32, pos as i32);
                    },

                    Some("BackSpace") => {
                        if !text.is_empty() {
                            let new_text = text[..text.len().saturating_sub(1)].to_string();
                            entry.set_text(&new_text);
                            entry.select_region(new_text.len() as i32, new_text.len() as i32);
                        }
                    },

                    _ => {}
                }
            }

            // Assume the user wants to continue typing if the key is... well, on the keyboard
            else if ALPHANUMERIC_SYMBOLIC_REGEX.is_match(&val.to_unicode().unwrap_or_default().to_string()) {
                entry.grab_focus_without_selecting();

                let mut text = entry.text().to_string();
                text.push(val.to_unicode().unwrap_or_default());
                entry.set_text(&text);
                entry.select_region(text.len() as i32, text.len() as i32);
            }
        }
    }));

    windows_box.add_controller(gesture::on_key_press({
        move |val, _| {
            if ALPHANUMERIC_SYMBOLIC_REGEX.is_match(&val.to_unicode().unwrap_or_default().to_string()) {
                entry.grab_focus_without_selecting();

                let mut text = entry.text().to_string();
                text.push(val.to_unicode().unwrap_or_default());
                entry.set_text(&text);
                entry.select_region(text.len() as i32, text.len() as i32);
            }
        }
    }));

    ipc::listen_for_messages_local(move |message| {
        if message.as_str() == "update_overview_windows" {
            // Tell the windows to update their contents
            frequent_window.update();
            recent_window.update();
        }
    });

    fullscreen
}