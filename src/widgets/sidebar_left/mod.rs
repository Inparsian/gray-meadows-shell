pub mod modules;

use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use futures_signals::signal::SignalExt;

use crate::{helpers::gesture, ipc, singletons::hyprland, widgets::common::tabs};

pub fn new(application: &libadwaita::Application) {
    let tabs = tabs::Tabs::new("sidebar-left-tab", true);
    tabs.current_tab.set(Some("color_picker".to_owned()));
    tabs.add_tab("translate", "translate".to_owned(), "g_translate");
    tabs.add_tab("color picker", "color_picker".to_owned(), "palette");

    view! {
        content = gtk4::Stack {
            set_css_classes: &["sidebar-left-content"],
            set_transition_type: gtk4::StackTransitionType::SlideLeftRight,
            set_transition_duration: 150,

            add_named: (&modules::translate::new(), Some("translate")),
            add_named: (&modules::color_picker::new(), Some("color_picker"))
        },

        left_sidebar_box = gtk4::Box {
            set_css_classes: &["left-sidebar-box"],
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 12,
            set_hexpand: true,
            set_vexpand: true,

            append: &tabs.widget,
            append: &content
        },

        window = gtk4::ApplicationWindow {
            set_css_classes: &["left-sidebar-window"],
            set_application: Some(application),
            init_layer_shell: (),
            set_namespace: Some("left-sidebar"),
            set_monitor: hyprland::get_active_monitor().as_ref(),
            set_keyboard_mode: KeyboardMode::Exclusive,
            set_layer: Layer::Overlay,
            set_anchor: (Edge::Left, true),
            set_anchor: (Edge::Right, false),
            set_anchor: (Edge::Top, true),
            set_anchor: (Edge::Bottom, true),

            set_child: Some(&left_sidebar_box)
        }
    };

    let current_tab_future = tabs.current_tab.signal_cloned().for_each(move |tab| {
        content.set_visible_child_name(tab.unwrap_or_default().as_str());

        async {}
    });

    window.add_controller(gesture::on_primary_up({
        let window = window.clone();

        move |_, x, y| {
            if window.is_visible() && !left_sidebar_box.allocation().contains_point(x as i32, y as i32) {
                window.hide();
            }
        }
    }));

    window.add_controller(gesture::on_key_press({
        let window = window.clone();

        move |val, _| {
            if val.name() == Some("Escape".into()) {
                window.hide();
            }
        }
    }));

    ipc::listen_for_messages_local(move |message| {
        if message.as_str() == "toggle_left_sidebar" {
            let monitor = hyprland::get_active_monitor();

            if window.is_visible() {
                window.hide();
            } else {
                window.set_monitor(monitor.as_ref());
                window.show();
            }
        }

        else if message.as_str() == "hide_left_sidebar" {
            window.hide();
        }
    });

    gtk4::glib::spawn_future_local(current_tab_future);
}