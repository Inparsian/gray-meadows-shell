mod item;
mod list;
mod modules;
mod windows;

use std::{cell::RefCell, rc::Rc, sync::LazyLock};
use freedesktop_desktop_entry::get_languages_from_env;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use regex::Regex;
use urlencoding::encode;

use crate::{
    helpers::gesture,
    ipc,
    singletons::{apps, hyprland},
    widgets::overview::{
        item::{OverviewSearchItem, OverviewSearchItemAction},
        list::{get_button_from_row, OverviewSearchList},
        modules::{input_without_extensions, validate_input, OverviewSearchModule},
        windows::{frequent::OverviewFrequentWindow, recent::OverviewRecentWindow}
    }
};

static MODULES: LazyLock<Vec<&(dyn OverviewSearchModule + Send + Sync)>> = LazyLock::new(|| vec![
    &modules::calculator::OverviewCalculatorModule,
    &modules::text::OverviewTextModule,
    &modules::terminal::OverviewTerminalModule,
    &modules::hashing::OverviewHashingModule
]);

static ALPHANUMERIC_SYMBOLIC_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new("^[a-zA-Z0-9 ~!@#$%^&*()_+\\-=\\[\\]{}|;':\",./<>?]+$").expect("Failed to compile alphanumeric symbolic regex")
});

fn generate_search_results(query: &str) -> Vec<OverviewSearchItem> {
    let mut results = Vec::new();

    // Iterate through all modules and collect results
    for module in MODULES.iter() {
        if validate_input(*module, query) {
            let input = input_without_extensions(*module, query);
            for item in module.run(&input) {
                results.push(item);
            }
        }
    }

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

pub fn new(application: &libadwaita::Application) {
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
            set_transition_type: gtk4::RevealerTransitionType::SlideDown,
            set_transition_duration: 250,
            set_reveal_child: false,

            libadwaita::Clamp {
                set_width_request: 0,
                set_maximum_size: 0,
                set_child: Some(&search_results.borrow().get_widget())
            },
        },

        windows = gtk4::Box {
            set_css_classes: &["overview-windows-box"],
            set_orientation: gtk4::Orientation::Horizontal,
            set_spacing: 8,
            set_hexpand: true,
            set_vexpand: true,

            append: &frequent_window.widget,
            append: &recent_window.widget
        },

        windows_revealer = gtk4::Revealer {
            set_transition_type: gtk4::RevealerTransitionType::SlideDown,
            set_transition_duration: 250,
            set_reveal_child: true,
            set_child: Some(&windows)
        },

        entry_box = gtk4::Box {
            set_css_classes: &["entry-box"],
            set_orientation: gtk4::Orientation::Horizontal,
            set_halign: gtk4::Align::Center
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
                        windows_revealer.set_reveal_child(true);
                        search_results_revealer.set_reveal_child(false);
                        entry_box.style_context().remove_class("entry-extended");
                    } else {
                        entry_prompt_revealer.set_reveal_child(false);
                        windows_revealer.set_reveal_child(false);
                        search_results_revealer.set_reveal_child(true);
                        entry_box.style_context().add_class("entry-extended");

                        let results = generate_search_results(&entry.text());

                        // Insert the new items into the search results list
                        let mut search_results_mut = search_results.borrow_mut();
                        for (i, item) in results.iter().enumerate() {
                            if let Some(existing_item) = search_results_mut.items.iter_mut().find(|i| i.smart_compare(item)) {
                                if item.exact_id_comp_has() {
                                    if item.query.is_some() && existing_item.query != item.query {
                                        existing_item.query = item.query.clone();
                                        existing_item.set_title_markup();
                                    }
                                } else {
                                    existing_item.set_title_label(&item.title);
                                    existing_item.set_action(item.action.clone());
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
        },

        window = gtk4::ApplicationWindow {
            set_css_classes: &["overview-window"],
            set_application: Some(application),
            init_layer_shell: (),
            set_monitor: hyprland::get_active_monitor().as_ref(),
            set_keyboard_mode: KeyboardMode::OnDemand,
            set_layer: Layer::Top,
            set_anchor: (Edge::Left, true),
            set_anchor: (Edge::Right, true),
            set_anchor: (Edge::Top, true),
            set_anchor: (Edge::Bottom, true),

            set_child: Some(&overview_box),
        }
    };

    entry_box.append(&entry_overlay);

    window.connect_unmap({
        let entry = entry.clone();
        move |_| { entry.set_text(""); }
    });

    window.connect_map({
        let entry = entry.clone();
        move |_| { entry.grab_focus(); }
    });

    window.add_controller(gesture::on_primary_up({
        let window = window.clone();
        move |_, x, y| {
            if window.is_visible() && !overview_box.allocation().contains_point(x as i32, y as i32) {
                window.hide();
            }
        }
    }));

    window.add_controller(gesture::on_key_press({
        let window = window.clone();
        let search_results = search_results.clone();

        move |val, _| {
            if val.name() == Some("Escape".into()) {
                window.hide();
            }

            // ListBoxRow steals the events for the Arrow Keys if it's focused, so
            // we can assume that it isn't focused if an event for Down is triggered
            // on the window
            else if val.name() == Some("Down".into()) {
                let first_child = search_results.borrow().get_widget().first_child();

                first_child.map(|child| child.downcast_ref::<gtk4::ListBoxRow>().map(|row| {
                    row.grab_focus();

                    if let Some(button) = get_button_from_row(row) {
                        button.grab_focus();
                    }
                }));
            }
        }
    }));

    {
        let search_results_widget = search_results.borrow().get_widget();
        search_results_widget.add_controller(gesture::on_key_press({
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
    }

    ipc::listen_for_messages_local(move |message| {
        if message.as_str() == "toggle_overview" {
            let monitor = hyprland::get_active_monitor();

            if window.is_visible() {
                window.hide();
            } else {
                window.set_monitor(monitor.as_ref());
                window.show();

                // Tell the windows to update their contents
                frequent_window.update();
                recent_window.update();
            }
        }

        else if message.as_str() == "hide_overview" {
            window.hide();
        }
    });
}