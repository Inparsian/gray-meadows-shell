use gtk4::prelude::*;
use futures_signals::signal::{Mutable, SignalExt};

pub fn new(
    label: &str,
    name: String,
    icon: &str,
    current_tab: &Mutable<Option<String>>
) -> gtk4::Button {
    view! {
        label_revealer = gtk4::Revealer {
            set_reveal_child: current_tab.get_cloned() == Some(name.clone()),
            set_transition_type: gtk4::RevealerTransitionType::SlideRight,
            set_transition_duration: 150,

            gtk4::Label {
                set_label: label,
                set_xalign: 0.0,
                set_ellipsize: gtk4::pango::EllipsizeMode::End,
                set_css_classes: &["sidebar-left-tab-label"]
            }
        },

        widget = gtk4::Button {
            set_css_classes: if current_tab.get_cloned() == Some(name.clone()) {
                &["sidebar-left-tab", "active"]
            } else {
                &["sidebar-left-tab"]
            },
            set_valign: gtk4::Align::Center,
            connect_clicked: {
                let current_tab = current_tab.clone();
                let name = name.clone();
                move |_| current_tab.set(Some(name.clone()))
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 2,

                gtk4::Label {
                    set_css_classes: &["sidebar-left-tab-icon"],
                    set_label: icon,
                    set_xalign: 0.5,
                    set_valign: gtk4::Align::Center,
                    set_halign: gtk4::Align::Center
                },

                append: &label_revealer
            }
        }
    }

    let current_tab_future = {
        let widget = widget.clone();

        current_tab.signal_cloned().for_each(move |tab| {
            widget.set_css_classes(if tab == Some(name.clone()) {
                &["sidebar-left-tab", "active"]
            } else {
                &["sidebar-left-tab"]
            });
            label_revealer.set_reveal_child(tab == Some(name.clone()));

            async {}
        })
    };

    gtk4::glib::spawn_future_local(current_tab_future);

    widget
}