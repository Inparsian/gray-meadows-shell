use gtk4::prelude::*;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum OverviewSearchItemAction {
    Launch(String),
    RunCommand(String),
    Copy(String),
    Custom(fn())
}

#[allow(dead_code)]
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
            widget = gtk4::Button {
                set_css_classes: &["overview-search-item"],

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
                }
            }
        }

        widget
    }
}