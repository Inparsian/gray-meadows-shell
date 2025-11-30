use std::{cell::RefCell, error::Error, rc::Rc};
use gtk4::prelude::*;
use relm4::RelmRemoveAllExt;

use crate::{
    singletons::g_translate::language::{self, Language, AUTO_LANG, LANGUAGES},
    widgets::windows::sidebar_left::modules::translate::{send_ui_event, set_source_language, set_target_language, subscribe_to_ui_events, LanguageSelectReveal, UiEvent}
};

const BUTTONS_PER_ROW: usize = 3;
const BUTTONS_PER_PAGE: usize = BUTTONS_PER_ROW * 12;

fn get_page_boxes(reveal_type: &LanguageSelectReveal, filter: Option<&str>) -> Vec<gtk4::Box> {
    let mut i = 0;
    let mut page_boxes: Vec<gtk4::Box> = Vec::new();
    let mut boxes: Vec<gtk4::Box> = Vec::new();
    let mut buttons: Vec<gtk4::Button> = Vec::new();

    let languages: Vec<Language> = if *reveal_type == LanguageSelectReveal::Source {
        let mut langs = vec![AUTO_LANG.clone()];
        langs.extend(LANGUAGES.iter().cloned());
        langs
    } else {
        LANGUAGES.clone()
    };

    for lang in languages {
        if filter.is_some_and(|f| !lang.name.to_lowercase().contains(f.to_lowercase().as_str())) {
            continue;
        }

        let button = gtk4::Button::new();
        button.set_css_classes(&["google-translate-language-select-button"]);
        button.set_hexpand(true);
        button.connect_clicked({
            let reveal_type = reveal_type.clone();
            let lang_name = lang.name.clone();

            move |_| {
                match reveal_type {
                    LanguageSelectReveal::Source => set_source_language(language::get_by_name(&lang_name)),
                    LanguageSelectReveal::Target => set_target_language(language::get_by_name(&lang_name)),
                    _ => unreachable!(),
                }

                send_ui_event(&UiEvent::LanguageSelectRevealChanged(LanguageSelectReveal::None));
            }
        });

        let button_label = gtk4::Label::new(Some(&lang.name));
        button_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        button.set_child(Some(&button_label));
        buttons.push(button);

        if i % BUTTONS_PER_ROW == BUTTONS_PER_ROW - 1 {
            boxes.push(gtk4::Box::new(gtk4::Orientation::Horizontal, 6));
            boxes.last().unwrap().set_homogeneous(true);
            boxes.last().unwrap().set_spacing(6);
            for button in &buttons {
                boxes.last().unwrap().append(button);
            }
            buttons.clear();
        }

        if i % BUTTONS_PER_PAGE == BUTTONS_PER_PAGE - 1 {
            page_boxes.push(gtk4::Box::new(gtk4::Orientation::Vertical, 6));
            page_boxes.last().unwrap().set_spacing(6);
            for box_ in &boxes {
                page_boxes.last().unwrap().append(box_);
            }
            boxes.clear();
        }

        i += 1;
    }
    
    if !buttons.is_empty() {
        boxes.push(gtk4::Box::new(gtk4::Orientation::Horizontal, 6));
        boxes.last_mut().unwrap().set_homogeneous(true);
        boxes.last_mut().unwrap().set_spacing(6);
        for button in buttons {
            boxes.last_mut().unwrap().append(&button);
        }
    }

    if !boxes.is_empty() {
        let page_box = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        page_box.set_css_classes(&["google-translate-language-select-page"]);
        page_box.set_spacing(6);
        page_box.set_vexpand(true);
        boxes.iter().for_each(|box_| page_box.append(box_));
        page_boxes.push(page_box);
        boxes.clear();
    }

    if !page_boxes.is_empty() {
        page_boxes
    } else {
        view! {
            no_results_box = gtk4::Box {
                set_css_classes: &["google-translate-language-select-page"],
                set_orientation: gtk4::Orientation::Vertical,
                set_spacing: 6,

                gtk4::Box {
                    set_css_classes: &["google-translate-language-select-row"],
                    set_homogeneous: true,
                    set_spacing: 6,

                    gtk4::Label {
                        set_label: "No languages found",
                        set_hexpand: true
                    }
                }
            }
        };

        vec![no_results_box]
    }
}

#[derive(Debug, Clone)]
pub struct LanguageSelectView {
    reveal_type: LanguageSelectReveal,
    current_page: Rc<RefCell<usize>>,
    page_boxes: Rc<RefCell<Vec<gtk4::Box>>>,
    pages_stack: gtk4::Stack,
    page_label: gtk4::Label,
    filter_entry: gtk4::Entry,
    filter_clear_button: gtk4::Button,
    widget: gtk4::Box
}

impl LanguageSelectView {
    pub fn new(reveal_type: LanguageSelectReveal) -> Self {
        let view = Self {
            reveal_type,
            current_page: Rc::new(RefCell::new(0)),
            page_boxes: Rc::new(RefCell::new(Vec::new())),
            pages_stack: gtk4::Stack::new(),
            page_label: gtk4::Label::new(Some("Page 0 of 0")),
            filter_clear_button: gtk4::Button::new(),
            filter_entry: gtk4::Entry::new(),
            widget: gtk4::Box::new(gtk4::Orientation::Vertical, 12)
        };

        view.pages_stack.set_css_classes(&["google-translate-language-select-stack"]);
        view.pages_stack.set_transition_type(gtk4::StackTransitionType::SlideLeftRight);
        view.pages_stack.set_transition_duration(250);

        view.page_label.set_css_classes(&["google-translate-language-select-page-label"]);

        view.filter_clear_button.set_css_classes(&["google-translate-language-select-filter-clear-button"]);
        view.filter_clear_button.set_visible(false);
        view.filter_clear_button.set_child(Some(&{
            let label = gtk4::Label::new(Some("clear"));
            label.set_css_classes(&["google-translate-language-select-filter-clear-label"]);
            label
        }));
        view.filter_clear_button.connect_clicked({
            let view = view.clone();
            move |_| view.clear_filter_entry()
        });

        view.filter_entry.set_css_classes(&["google-translate-language-select-filter-entry"]);
        view.filter_entry.set_placeholder_text(Some("Filter languages..."));
        view.filter_entry.set_hexpand(true);
        view.filter_entry.connect_changed({
            let view = view.clone();
            let filter_clear_button = view.filter_clear_button.clone();
            move |entry| {
                let _ = view.update_page_boxes(Some(entry.text().as_str()));
                filter_clear_button.set_visible(!entry.text().is_empty());
            }
        });

        view.widget.set_css_classes(&["google-translate-language-select-box"]);
        view.widget.set_hexpand(true);
        view.widget.append(&{
            let label = gtk4::Label::new(Some(if view.reveal_type == LanguageSelectReveal::Source {
                "Source Language"
            } else {
                "Target Language"
            }));

            label.set_css_classes(&["google-translate-language-select-label"]);
            label
        });

        view.widget.append(&{
            let filter_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
            filter_box.set_css_classes(&["google-translate-language-select-filter-box"]);
            filter_box.append(&view.filter_entry);
            filter_box.append(&view.filter_clear_button);
            filter_box
        });

        view.widget.append(&{
            let clamp = libadwaita::Clamp::new();
            clamp.set_width_request(400);
            clamp.set_maximum_size(400);
            clamp.set_child(Some(&view.pages_stack));
            clamp.set_unit(libadwaita::LengthUnit::Px);
            clamp
        });

        view.widget.append(&{
            let nav_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
            nav_box.set_homogeneous(true);
            nav_box.append(&{
                let prev_button = gtk4::Button::new();
                prev_button.set_css_classes(&["google-translate-language-select-nav-button"]);
                prev_button.set_label("Previous");
                prev_button.connect_clicked({
                    let view = view.clone();
                    move |_| {
                        // .max(1) prevents substraction overflow.
                        let _ = view.set_page(view.get_current_page().max(1) - 1);
                    }
                });
                prev_button
            });

            nav_box.append(&view.page_label);

            nav_box.append(&{
                let next_button = gtk4::Button::new();
                next_button.set_css_classes(&["google-translate-language-select-nav-button"]);
                next_button.set_label("Next");
                next_button.connect_clicked({
                    let view = view.clone();
                    move |_| {
                        let _ = view.set_page(view.get_current_page() + 1);
                    }
                });
                next_button
            });

            nav_box
        });

        let _ = view.update_page_boxes(None);

        // Start our event receiver task
        let receiver = subscribe_to_ui_events();
        gtk4::glib::spawn_future_local({
            let view = view.clone();
            async move {
                while let Ok(event) = receiver.recv().await {
                    if let UiEvent::LanguageSelectRevealChanged(reveal) = event {
                        if reveal != view.reveal_type {
                            continue;
                        }

                        view.clear_filter_entry();
                    }
                }
            }
        });

        view
    }

    pub fn get_widget(&self) -> &gtk4::Box {
        &self.widget
    }

    pub fn get_current_page(&self) -> usize {
        self.current_page.try_borrow().map_or(0, |page| *page)
    }

    pub fn get_total_pages(&self) -> usize {
        self.page_boxes.try_borrow().map_or(0, |boxes| boxes.len())
    }

    pub fn set_page(&self, page: usize) -> Result<(), Box<dyn Error>> {
        {
            let mut current_page = self.current_page.try_borrow_mut()?;
            *current_page = page.clamp(0, self.get_total_pages() - 1);
            self.pages_stack.set_visible_child_name(&format!("page_{}", *current_page));
        }

        self.update_page_label();
        Ok(())
    }

    pub fn update_page_label(&self) {
        self.page_label.set_label(&format!("Page {} of {}", self.get_current_page() + 1, self.get_total_pages()));
    }

    pub fn update_page_boxes(&self, filter: Option<&str>) -> Result<(), Box<dyn Error>> {
        let new_boxes = get_page_boxes(&self.reveal_type, filter);
        self.page_boxes.try_borrow_mut()?.clear();
        self.page_boxes.try_borrow_mut()?.extend(new_boxes);
    
        self.pages_stack.remove_all();
        for (i, page_box) in self.page_boxes.borrow().iter().enumerate() {
            self.pages_stack.add_named(page_box, Some(&format!("page_{}", i)));
        }

        self.set_page(0)
    }

    pub fn clear_filter_entry(&self) {
        self.filter_entry.set_text("");
        self.filter_entry.grab_focus();
    }
}