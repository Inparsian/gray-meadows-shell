use std::{cell::RefCell, rc::Rc};
use gtk4::prelude::*;
use futures_signals::signal::{Mutable, SignalExt as _};

use crate::utils::gesture;

pub enum TabSize {
    Tiny,
    Normal,
    Large
}

impl TabSize {
    pub fn to_class_name(&self) -> &'static str {
        match self {
            TabSize::Tiny => "tiny",
            TabSize::Normal => "normal",
            TabSize::Large => "large"
        }
    }
}

pub struct Tab {
    pub name: String,
    pub widget: gtk4::Button,
}

pub struct TabGroupBuilder<'a> {
    tabs: &'a Tabs,
    spacing: i32,
    class_name: Option<&'a str>,
    hexpand: bool,
    vexpand: bool,
}

impl<'a> TabGroupBuilder<'a> {
    fn new(tabs: &'a Tabs) -> Self {
        Self {
            tabs,
            spacing: Default::default(),
            class_name: Default::default(),
            hexpand: Default::default(),
            vexpand: Default::default(),
        }
    }
    
    pub fn spacing(mut self, spacing: i32) -> Self {
        self.spacing = spacing;
        self
    }
    
    pub fn class_name(mut self, class_name: &'a str) -> Self {
        self.class_name = Some(class_name);
        self
    }
    
    pub fn hexpand(mut self, hexpand: bool) -> Self {
        self.hexpand = hexpand;
        self
    }
    
    pub fn vexpand(mut self, vexpand: bool) -> Self {
        self.vexpand = vexpand;
        self
    }
    
    pub fn build(self) -> gtk4::Box {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, self.spacing);
        if let Some(class_name) = self.class_name.as_ref() {
            widget.set_css_classes(&[class_name]);
        }
        widget.set_hexpand(self.hexpand);
        widget.set_vexpand(self.vexpand);
        widget.append(&self.tabs.select);
        widget.append(&self.tabs.stack);
        widget
    }
}

pub struct Tabs {
    size: TabSize,
    only_current_tab_visible: bool,
    pub current_tab: Mutable<Option<String>>,
    pub items: Rc<RefCell<Vec<Tab>>>,
    pub stack: gtk4::Stack,
    pub select: gtk4::Box,
}

impl Tabs {
    pub fn new(size: TabSize, only_current_tab_visible: bool, class_name: Option<&str>) -> Self {
        let select = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);        
        let stack = gtk4::Stack::builder()
            .css_classes([class_name.unwrap_or("tabs-stack")])
            .transition_type(gtk4::StackTransitionType::SlideLeftRight)
            .transition_duration(150)
            .build();

        let tabs = Self {
            size,
            only_current_tab_visible,
            current_tab: Mutable::new(None),
            items: Rc::new(RefCell::new(Vec::new())),
            stack,
            select,
        };

        tabs.select.add_controller(gesture::on_vertical_scroll({
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
        
        glib::spawn_future_local({
            let stack = tabs.stack.clone();
            signal_cloned!(tabs.current_tab, (tab) {
                stack.set_visible_child_name(tab.as_deref().unwrap_or_default());
            })
        });

        tabs
    }
    
    pub fn group(&self) -> TabGroupBuilder<'_> {
        TabGroupBuilder::new(self)
    }

    pub fn add_tab(&self, label: &str, name: &str, icon: Option<&str>, widget: &impl IsA<gtk4::Widget>) {
        let tab = Tab {
            name: name.to_owned(),
            widget: self.create_tab_widget(label, name.to_owned(), icon, &self.current_tab)
        };

        self.select.append(&tab.widget);
        self.items.borrow_mut().push(tab);
        self.stack.add_named(widget, Some(name));
    }
    
    pub fn set_current_tab(&self, name: Option<&str>) {
        self.current_tab.set(name.map(|s| s.to_owned()));
    }

    fn create_tab_widget(&self, label: &str, name: String, icon: Option<&str>, current_tab: &Mutable<Option<String>>) -> gtk4::Button {
        let tab_class_name = self.size.to_class_name();
        let label_widget: gtk4::Widget = {
            let label = gtk4::Label::builder()
                .label(label)
                .css_classes(["tab-label".to_owned()])
                .xalign(0.0)
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
                set_css_classes: &["tab", tab_class_name],
                set_valign: gtk4::Align::Center,
                connect_clicked: clone!(
                    #[strong] current_tab,
                    #[strong] name,
                    move |_| current_tab.set(Some(name.clone()))
                ),

                gtk4::Box {
                    set_orientation: gtk4::Orientation::Horizontal,

                    gtk4::Label {
                        set_css_classes: &["tab-icon"],
                        set_visible: icon.is_some(),
                        set_label: icon.unwrap_or_default(),
                        set_xalign: 0.5,
                        set_valign: gtk4::Align::Center,
                        set_halign: gtk4::Align::Center
                    },

                    append: &label_widget
                }
            }
        };

        glib::spawn_future_local({
            let widget = widget.clone();
            let only_current_tab_visible = self.only_current_tab_visible;

            signal_cloned!(current_tab, (tab) {
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
            })
        });

        widget
    }
}