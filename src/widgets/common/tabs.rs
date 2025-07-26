use std::{cell::RefCell, rc::Rc};
use gtk4::prelude::*;
use futures_signals::signal::{Mutable, SignalExt};

use crate::helpers::gesture;

#[derive(Clone)]
pub struct Tab {
    pub name: String,
    pub widget: gtk4::Button
}

#[derive(Clone)]
pub struct Tabs {
    pub tab_class_name: String,
    pub only_current_tab_visible: bool,
    pub current_tab: Mutable<Option<String>>,
    pub items: Rc<RefCell<Vec<Tab>>>,
    pub widget: gtk4::Box
}

impl Tabs {
    pub fn new(tab_class_name: &str, only_current_tab_visible: bool) -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);

        let tabs = Self {
            tab_class_name: tab_class_name.to_owned(),
            only_current_tab_visible,
            current_tab: Mutable::new(None),
            items: Rc::new(RefCell::new(Vec::new())),
            widget
        };

        tabs.widget.add_controller(gesture::on_vertical_scroll({
            let current_tab = tabs.current_tab.clone();
            let items = tabs.items.clone();

            move |dy| {
                let items_borrow = items.borrow();
                let current_tab_index = items_borrow.iter().position(|tab| tab.name == current_tab.get_cloned().unwrap_or_default()).unwrap_or(0);

                if dy < 0.0 && current_tab_index > 0 {
                    current_tab.set(Some(items_borrow[current_tab_index - 1].name.clone()));
                } else if dy > 0.0 && current_tab_index < items_borrow.len() - 1 {
                    current_tab.set(Some(items_borrow[current_tab_index + 1].name.clone()));
                }
            }
        }));

        tabs
    }

    pub fn add_tab(&self, label: &str, name: String, icon: &str) {
        let widget = self.create_tab_widget(label, name.clone(), icon, &self.current_tab);
        let tab = Tab {
            name,
            widget
        };

        self.widget.append(&tab.widget);
        self.items.borrow_mut().push(tab);
    }

    pub fn create_tab_widget(&self, label: &str, name: String, icon: &str, current_tab: &Mutable<Option<String>>) -> gtk4::Button {
        let tab_class_name = self.tab_class_name.clone();
        let label_widget: gtk4::Widget = {
            let label = gtk4::Label::builder()
                .label(label)
                .css_classes([format!("{tab_class_name}-label")])
                .xalign(0.0)
                .ellipsize(gtk4::pango::EllipsizeMode::End)
                .build();

            if self.only_current_tab_visible {
                let label_revealer = gtk4::Revealer::builder()
                    .reveal_child(current_tab.get_cloned() == Some(name.clone()))
                    .transition_type(gtk4::RevealerTransitionType::SlideRight)
                    .transition_duration(150)
                    .child(&label)
                    .build();

                label_revealer.upcast()
            } else {
                label.upcast()
            }
        };

        view! {
            widget = gtk4::Button {
                set_css_classes: &[tab_class_name.as_str()],
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
                        set_css_classes: &[&format!("{tab_class_name}-icon")],
                        set_label: icon,
                        set_xalign: 0.5,
                        set_valign: gtk4::Align::Center,
                        set_halign: gtk4::Align::Center
                    },

                    append: &label_widget
                }
            }
        };

        let current_tab_future = {
            let widget = widget.clone();
            let only_current_tab_visible = self.only_current_tab_visible;

            current_tab.signal_cloned().for_each(move |tab| {
                if tab.as_ref() == Some(&name) {
                    widget.add_css_class("active");
                } else {
                    widget.remove_css_class("active");
                }

                if only_current_tab_visible {
                    let label_revealer = label_widget.downcast_ref::<gtk4::Revealer>();
                    if let Some(r) = label_revealer {
                        r.set_reveal_child(tab == Some(name.clone()));
                    }
                }
            
                async {}
            })
        };

        gtk4::glib::spawn_future_local(current_tab_future);

        widget
    }
}