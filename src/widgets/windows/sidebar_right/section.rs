use gtk4::prelude::*;

pub struct SideRightSection {
    pub bx: gtk4::Box,
    pub title: gtk4::Label,
    pub icon: gtk4::Label,
    pub content: gtk4::Label,
}

impl SideRightSection {
    pub fn new(
        title_str: &str,
        icon_str: &str,
        content_str: &str,
    ) -> Self {
        view! {
            title = gtk4::Label {
                set_label: title_str,
                set_css_classes: &["sidebar-right-section-title"],
                set_xalign: 0.0,
                set_valign: gtk4::Align::Center,
                set_vexpand: true,
            },

            icon = gtk4::Label {
                set_label: icon_str,
                set_css_classes: &["sidebar-right-section-icon"],
                set_xalign: 0.0,
                set_valign: gtk4::Align::Center,
                set_vexpand: true,
            },

            content = gtk4::Label {
                set_label: content_str,
                set_css_classes: &["sidebar-right-section-content"],
                set_xalign: 0.0,
                set_valign: gtk4::Align::Center,
                set_vexpand: true,
            },

            bx = gtk4::Box {
                set_css_classes: &["sidebar-right-section-box"],
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 0,
                set_hexpand: true,

                append: &icon,

                gtk4::Box {
                    set_orientation: gtk4::Orientation::Vertical,
                    set_spacing: 0,
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: gtk4::Align::Start,

                    append: &title,
                    append: &content
                },

                gtk4::Label {
                    set_css_classes: &["sidebar-right-section-arrow"],
                    set_label: "chevron_right",
                    set_xalign: 1.0,
                    set_halign: gtk4::Align::End,
                    set_valign: gtk4::Align::Center,
                    set_hexpand: true,
                    set_vexpand: true,
                }
            }
        };

        SideRightSection {
            bx,
            title,
            icon,
            content,
        }
    }

    pub fn set_toggled(&self, toggled: bool) {
        if toggled {
            self.bx.add_css_class("toggled");
        } else {
            self.bx.remove_css_class("toggled");
        }
    }

    pub fn set_content(&self, content_str: &str) {
        self.content.set_label(content_str);
    }

    pub fn set_icon(&self, icon_str: &str) {
        self.icon.set_label(icon_str);
    }

    pub fn set_title(&self, title_str: &str) {
        self.title.set_label(title_str);
    }
}