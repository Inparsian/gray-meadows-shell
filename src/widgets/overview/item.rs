use gtk4::prelude::*;

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
        relm4_macros::view! {
            action_slide_revealer = gtk4::Revealer {
                set_transition_type: gtk4::RevealerTransitionType::SlideLeft,
                set_transition_duration: 175,
                set_reveal_child: false,

                gtk4::Label {
                    set_css_classes: &["overview-search-item-action"],
                    set_label: &self.action_text,
                    set_xalign: 1.0
                }
            },

            widget = gtk4::Button {
                set_css_classes: &["overview-search-item"],

                connect_clicked: {
                    let action = self.action.clone();
                    move |_| {
                        match &action {
                            // Launch and RunCommand will share the same behavior for now, however in the
                            // future, Launch will enable gray-meadows-shell to internally track the most
                            // launched applications, hence why it is separated from RunCommand.
                            OverviewSearchItemAction::Launch(command) | OverviewSearchItemAction::RunCommand(command) => {
                                println!("Running command: {}", command);
                            }

                            OverviewSearchItemAction::Copy(text) => {
                                println!("Copying to clipboard: {}", text);
                            }

                            OverviewSearchItemAction::Custom(func) => func(),
                        }
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
                        set_icon_name: Some(&self.icon),
                        set_css_classes: &["overview-search-item-icon"],
                    },

                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Vertical,
                        set_hexpand: true,

                        gtk4::Label {
                            set_label: self.subtitle.as_ref().unwrap_or(&"".to_string()).as_str(),
                            set_css_classes: &["overview-search-item-subtitle"],
                            set_xalign: 0.0
                        },

                        gtk4::Label {
                            set_label: &self.title,
                            set_css_classes: &["overview-search-item-title"],
                            set_xalign: 0.0
                        }
                    },

                    append: &action_slide_revealer
                }
            }
        }

        widget
    }
}