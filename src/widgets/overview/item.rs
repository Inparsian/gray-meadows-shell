use gtk4::prelude::*;

use crate::{helpers::process, ipc, singletons::apps::pixbuf::get_pixbuf_or_fallback};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum OverviewSearchItemAction {
    Launch(String),
    RunCommand(String),
    Copy(String),
    Custom(fn())
}

#[derive(Clone, Debug)]
pub struct OverviewSearchItem {
    pub title: String,
    pub subtitle: Option<String>,
    pub icon: String,
    pub action_text: String,
    pub action: OverviewSearchItemAction,
}

impl OverviewSearchItem {
    pub fn build(&self) -> gtk4::Button {
        let icon = get_pixbuf_or_fallback(&self.icon, "emote-love");

        relm4_macros::view! {
            action_slide_revealer = gtk4::Revealer {
                set_transition_type: gtk4::RevealerTransitionType::SlideLeft,
                set_transition_duration: 175,
                set_reveal_child: false,

                gtk4::Label {
                    set_css_classes: &["overview-search-item-action"],
                    set_label: &self.action_text,
                    set_xalign: 1.0,
                    set_ellipsize: gtk4::pango::EllipsizeMode::Start
                }
            },

            widget = gtk4::Button {
                set_css_classes: &["overview-search-item"],

                connect_clicked: {
                    let action = self.action.clone();
                    move |_| {
                        match &action {
                            OverviewSearchItemAction::Launch(command) => {
                                process::launch(command);
                            }
                            
                            OverviewSearchItemAction::RunCommand(command) => {
                                std::thread::spawn({
                                    let command = command.clone();

                                    move || {
                                        let _output = std::process::Command::new("bash")
                                            .arg("-c")
                                            .arg(command)
                                            .output();
                                    }
                                });
                            },

                            // TODO: Do this without wl-copy?
                            OverviewSearchItemAction::Copy(text) => {
                                std::thread::spawn({
                                    let text = text.clone();

                                    move || {
                                        let _output = std::process::Command::new("wl-copy")
                                            .arg(text)
                                            .output();
                                    }
                                });
                            }

                            OverviewSearchItemAction::Custom(func) => func(),
                        }

                        // Hide the overview after clicking an item
                        let _ = ipc::client::send_message("hide_overview");
                    }
                },

                connect_has_focus_notify: {
                    let action_slide_revealer = action_slide_revealer.clone();

                    move |button| {
                        action_slide_revealer.set_reveal_child(button.has_focus());
                    }
                },

                gtk4::Box {
                    set_css_classes: &["overview-search-item-box"],
                    set_orientation: gtk4::Orientation::Horizontal,
                    set_hexpand: true,

                    gtk4::Image {
                        set_from_pixbuf: icon.as_ref(),
                        set_pixel_size: 24,
                        set_css_classes: &["overview-search-item-icon"],
                    },

                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Vertical,
                        set_valign: gtk4::Align::Center,
                        set_hexpand: true,

                        gtk4::Label {
                            set_label: self.subtitle.as_ref().unwrap_or(&String::new()).as_str(),
                            set_visible: self.subtitle.is_some(),
                            set_css_classes: &["overview-search-item-subtitle"],
                            set_xalign: 0.0,
                            set_ellipsize: gtk4::pango::EllipsizeMode::End
                        },

                        gtk4::Label {
                            set_label: &self.title,
                            set_css_classes: &["overview-search-item-title"],
                            set_xalign: 0.0,
                            set_ellipsize: gtk4::pango::EllipsizeMode::End
                        }
                    },

                    append: &action_slide_revealer
                }
            }
        }

        widget
    }
}