use std::{cell::RefCell, rc::Rc};
use gtk4::prelude::*;

use crate::{widgets::windows::Window, helpers::{matching, scss}, singletons::apps::{self, pixbuf::get_pixbuf_or_fallback}};

pub static ITEM_ANIMATION_DURATION: u32 = 175;

// The IDs that should be compared exactly with eq, and whose corresponding results
// are not expected to change, except for the query.
// This is used to avoid unnecessary widget re-creation for items that shouldn't be
// re-created, such as results for modules such as calculator or text.
static EXACT_ID_COMP: [&str; 1] = [
    "application-result"
];

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OverviewSearchItemAction {
    Launch(String),
    RunCommand(String),
    Copy(String),
    Custom(fn())
}

#[derive(Clone, Debug)]
pub struct OverviewSearchItem {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub icon: String,
    pub action_text: String,
    pub action: Rc<RefCell<OverviewSearchItemAction>>,
    pub query: Option<String>,
    row: gtk4::ListBoxRow,
    widget: gtk4::Revealer,
    title_label: gtk4::Label
}

impl OverviewSearchItem {
    pub fn new(
        id: String,
        title: String,
        subtitle: Option<String>,
        icon: String,
        action_text: String,
        action: OverviewSearchItemAction,
        query: Option<String>
    ) -> Self {
        let icon_pixbuf = get_pixbuf_or_fallback(&icon, "emote-love");
        let action = Rc::new(RefCell::new(action));

        view! {
            title_label = gtk4::Label {
                set_label: &title,
                set_css_classes: &["overview-search-item-title"],
                set_xalign: 0.0,
                set_ellipsize: gtk4::pango::EllipsizeMode::End
            },
            
            action_slide_revealer = gtk4::Revealer {
                set_transition_type: gtk4::RevealerTransitionType::SlideLeft,
                set_transition_duration: 175,
                set_reveal_child: false,

                gtk4::Label {
                    set_css_classes: &["overview-search-item-action"],
                    set_label: &action_text,
                    set_xalign: 1.0,
                    set_ellipsize: gtk4::pango::EllipsizeMode::Start
                }
            },

            button = gtk4::Button {
                set_css_classes: &["overview-search-item"],

                connect_clicked: {
                    let action = action.clone();
                    move |_| if let Ok(action) = action.try_borrow() {
                        run_action(&action);
                    }
                },
            
                connect_has_focus_notify: {
                    let action_slide_revealer = action_slide_revealer.clone();
                    move |button| action_slide_revealer.set_reveal_child(button.has_focus())
                },
            
                gtk4::Box {
                    set_css_classes: &["overview-search-item-box"],
                    set_orientation: gtk4::Orientation::Horizontal,
                    set_hexpand: true,
                
                    gtk4::Image {
                        set_from_pixbuf: icon_pixbuf.as_ref(),
                        set_pixel_size: 24,
                        set_css_classes: &["overview-search-item-icon"],
                    },
                
                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Vertical,
                        set_valign: gtk4::Align::Center,
                        set_hexpand: true,
                    
                        gtk4::Label {
                            set_label: subtitle.as_ref().unwrap_or(&String::new()).as_str(),
                            set_visible: subtitle.is_some(),
                            set_css_classes: &["overview-search-item-subtitle"],
                            set_xalign: 0.0,
                            set_ellipsize: gtk4::pango::EllipsizeMode::End
                        },

                        append: &title_label
                    },
                
                    append: &action_slide_revealer
                }
            },

            widget = gtk4::Revealer {
                set_transition_type: gtk4::RevealerTransitionType::SlideDown,
                set_css_classes: &["overview-search-item-revealer"],
                set_transition_duration: ITEM_ANIMATION_DURATION,
                set_reveal_child: false,
                set_child: Some(&button)
            },

            row = gtk4::ListBoxRow {
                set_child: Some(&widget),
                set_css_classes: &["overview-search-item-row"]
            }
        }

        let item = Self {
            id,
            title,
            subtitle,
            icon,
            action_text,
            action,
            query,
            row,
            widget,
            title_label
        };

        item.set_title_markup();
        item
    }

    pub fn get_row(&self) -> gtk4::ListBoxRow {
        self.row.clone()
    }

    pub fn eq(&self, other: &Self) -> bool {
        self.id == other.id &&
        self.title == other.title &&
        self.subtitle == other.subtitle &&
        self.icon == other.icon &&
        self.action_text == other.action_text &&
        match (self.action.try_borrow(), other.action.try_borrow()) {
            (Ok(a), Ok(b)) => a.clone() == b.clone(),
            _ => false
        }
    }

    pub fn id_eq(&self, other: &Self) -> bool {
        self.id == other.id
    }

    pub fn exact_id_comp_has(&self) -> bool {
        EXACT_ID_COMP.contains(&self.id.as_str())
    }

    pub fn smart_compare(&self, other: &Self) -> bool {
        // Results that should be compared exactly with eq
        if self.exact_id_comp_has() {
            self.eq(other)
        } else {
            self.id_eq(other)
        }
    }

    pub fn reveal(&self) {
        self.widget.set_reveal_child(true);
        self.widget.add_css_class("revealed");
    }

    pub fn hide(&self) {
        self.widget.set_reveal_child(false);
        self.widget.remove_css_class("revealed");
    }

    pub fn set_title_label(&mut self, title: &str) {
        self.title = title.to_owned();
        self.title_label.set_label(&self.title);
        self.set_title_markup();
    }

    pub fn set_title_markup(&self) {
        if let Some(query) = &self.query {
            // Build the markup for our query using lazy match indices
            let indices: Vec<(usize, usize)> = matching::lazy_match_indices(
                &self.title.to_lowercase(),
                &query.to_lowercase()
            );

            let mut chars: Vec<String> = Vec::new();

            for (i, c) in self.title.chars().enumerate() {
                chars.push(if indices.iter().any(|(start, _)| *start == i) {
                    format!(
                        "<b>{}</b>",
                        scss::escape_html(c)
                    )
                } else {
                    scss::get_color("foreground-color-select").map_or(
                        scss::escape_html(c),
                        |color| format!(
                            "<span foreground=\"{}\">{}</span>",
                            color.as_hex(),
                            scss::escape_html(c)
                        )
                    )
                });
            }

            self.title_label.set_markup(&chars.join(""));
        }
    }

    pub fn set_action(&self, action: OverviewSearchItemAction) {
        if let Ok(mut act) = self.action.try_borrow_mut() {
            *act = action;
        }
    }
}

pub fn run_action(action: &OverviewSearchItemAction) {
    match action {
        OverviewSearchItemAction::Launch(command) => apps::launch_and_track(command),

        OverviewSearchItemAction::RunCommand(command) => {
            std::thread::spawn({
                let command = command.clone();
            
                move || std::process::Command::new("bash")
                    .arg("-c")
                    .arg(command)
                    .output()
            });
        },
    
        // TODO: Do this without wl-copy?
        OverviewSearchItemAction::Copy(text) => {
            std::thread::spawn({
                let text = text.clone();
            
                move || std::process::Command::new("wl-copy")
                    .arg(text)
                    .output()
            });
        }
    
        OverviewSearchItemAction::Custom(func) => func()
    }

    // Hide the overview after clicking an item
    Window::Overview.hide();
}