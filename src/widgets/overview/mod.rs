mod item;
mod modules;

use freedesktop_desktop_entry::get_languages_from_env;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use relm4::RelmRemoveAllExt;
use urlencoding::encode;

use crate::{
    helpers::gesture,
    ipc,
    singletons::{apps, hyprland},
    widgets::overview::item::{OverviewSearchItem, OverviewSearchItemAction}
};

fn generate_search_results(query: &str) -> Vec<OverviewSearchItem> {
    let mut results = Vec::new();

    // Filter and weigh the applications based on the query
    let locales = get_languages_from_env();
    let desktops = apps::query_desktops(query);
    for i in 0..8 {
        if let Some(entry) = desktops.get(i) {
            let entry = &entry.entry;

            results.push(OverviewSearchItem {
                title: entry.name(&locales).unwrap_or_default().to_string(),
                subtitle: None,
                icon: entry.icon().map(|icon| icon.to_string()).unwrap_or_default(),
                action: OverviewSearchItemAction::Launch(entry.exec().unwrap_or_default().to_string()),
                action_text: "launch".to_string(),
            });
        }
    }

    // web search as final fallback
    results.push(OverviewSearchItem {
        title: query.to_string(),
        subtitle: Some("Search the web".to_string()),
        icon: "search".to_string(),
        action: OverviewSearchItemAction::RunCommand(format!("xdg-open https://duckduckgo.com/?q={}", encode(query))),
        action_text: "search".to_string(),
    });

    results
}

pub fn new(application: &libadwaita::Application) {
    let search_results = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    search_results.set_css_classes(&["overview-search-results"]);
    search_results.set_height_request(200);
    
    relm4_macros::view! {
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
            set_transition_duration: 175,
            set_reveal_child: false,

            set_child: Some(&search_results),
        },

        entry = gtk4::Entry {
            set_css_classes: &["entry-prompt"],
            set_has_frame: false,

            connect_activate: {
                let search_results = search_results.clone();

                move |entry| {
                    let text = entry.text().to_string();
                    if !text.is_empty() {
                        search_results.first_child().map(|child| child.activate());
                    }
                }
            },

            connect_changed: {
                let entry_prompt_revealer = entry_prompt_revealer.clone();
                let search_results = search_results.clone();
                let search_results_revealer = search_results_revealer.clone();

                move |entry| {
                    if entry.text().is_empty() {
                        entry_prompt_revealer.set_reveal_child(true);
                        search_results_revealer.set_reveal_child(false);
                        entry.style_context().remove_class("entry-extended");
                    } else {
                        entry_prompt_revealer.set_reveal_child(false);
                        search_results_revealer.set_reveal_child(true);
                        entry.style_context().add_class("entry-extended");

                        // Clear previous results
                        search_results.remove_all();

                        for item in generate_search_results(&entry.text()) {
                            search_results.append(&item.build());
                        }
                    }
                }
            }
        },

        entry_box = gtk4::Box {
            set_css_classes: &["entry-box"],
            set_orientation: gtk4::Orientation::Horizontal,
            set_hexpand: true,

            append: &entry,
        },

        overview_box = gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 8,
            set_halign: gtk4::Align::Center,
            set_valign: gtk4::Align::Center,
            set_hexpand: true,

            gtk4::Overlay {
                set_child: Some(&entry_box),
                add_overlay: &entry_prompt_revealer,
            },

            append: &search_results_revealer,
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

    window.add_controller(gesture::on_primary_click({
        let window = window.clone();
        let entry = entry.clone();
        let overview_box = overview_box.clone();

        move |_, x, y| {
            if window.is_visible() && !overview_box.allocation().contains_point(x as i32, y as i32) {
                window.hide();
                entry.set_text("");
            }
        }
    }));

    window.add_controller(gesture::on_key_press({
        let window = window.clone();
        let entry = entry.clone();

        move |val, _| {
            if val.name() == Some("Escape".into()) {
                window.hide();
                entry.set_text("");
            }
        }
    }));

    ipc::listen_for_messages_local(move |message| {
        if message.as_str() == "toggle_overview" {
            let monitor = hyprland::get_active_monitor();

            if window.is_visible() {
                window.hide();
                entry.set_text("");
            } else {
                window.set_monitor(monitor.as_ref());
                window.show();
                entry.grab_focus();
            }
        }

        else if message.as_str() == "hide_overview" {
            window.hide();
            entry.set_text("");
        }
    });
}