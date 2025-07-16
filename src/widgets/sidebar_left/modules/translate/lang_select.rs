use std::{rc::Rc, sync::Mutex};
use gtk4::prelude::*;
use relm4::RelmRemoveAllExt;

use crate::{
    singletons::g_translate::language::{self, Language, AUTO_LANG, LANGUAGES},
    widgets::sidebar_left::modules::translate::{send_ui_event, set_source_language, set_target_language, subscribe_to_ui_events, LanguageSelectReveal, UiEvent}
};

const BUTTONS_PER_ROW: usize = 3;
const BUTTONS_PER_PAGE: usize = BUTTONS_PER_ROW * 12;

fn get_page_boxes(reveal_type: LanguageSelectReveal, filter: Option<&str>) -> Vec<gtk4::Box> {
    let mut i = 0;
    let mut page_boxes: Vec<gtk4::Box> = Vec::new();
    let mut boxes: Vec<gtk4::Box> = Vec::new();
    let mut buttons: Vec<gtk4::Button> = Vec::new();

    let languages: Vec<Language> = if reveal_type == LanguageSelectReveal::Source {
        let mut langs = vec![AUTO_LANG.clone()];
        langs.extend(LANGUAGES.iter().cloned());
        langs
    } else {
        LANGUAGES.clone()
    };

    for lang in languages {
        if let Some(filter) = filter {
            if !lang.name.to_lowercase().contains(filter.to_lowercase().as_str()) {
                continue;
            }
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

                send_ui_event(UiEvent::LanguageSelectRevealChanged(LanguageSelectReveal::None));
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
        page_boxes.push(gtk4::Box::new(gtk4::Orientation::Vertical, 6));
        page_boxes.last_mut().unwrap().set_spacing(6);
        for box_ in &boxes {
            page_boxes.last_mut().unwrap().append(box_);
        }
        boxes.clear();
    }

    if !page_boxes.is_empty() {
        page_boxes
    } else {
        relm4_macros::view! {
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

pub fn new(reveal_type: LanguageSelectReveal) -> gtk4::Box {
    let current_page: Rc<Mutex<usize>> = Rc::new(Mutex::new(0));
    let page_boxes: Rc<Mutex<Vec<gtk4::Box>>> = {
        let reveal_type = reveal_type.clone();

        Rc::new(Mutex::new(
            get_page_boxes(reveal_type, None)
        ))
    };

    relm4_macros::view! {
        pages_stack = gtk4::Stack {
            set_css_classes: &["google-translate-language-select-stack"],
            set_transition_type: gtk4::StackTransitionType::SlideLeftRight,
            set_transition_duration: 250
        },

        page_label = gtk4::Label {
            set_css_classes: &["google-translate-language-select-page-label"],
            set_label: &format!(
                "Page {} of {}",
                *current_page.lock().unwrap() + 1,
                page_boxes.lock().unwrap().len()
            ),
        },

        filter_clear_button = gtk4::Button {
            set_css_classes: &["google-translate-language-select-filter-clear-button"],
            set_visible: false,
            connect_clicked: {
                let filter_entry = filter_entry.clone();
                move |_| filter_entry.set_text("")
            },

            gtk4::Label {
                set_css_classes: &["google-translate-language-select-filter-clear-label"],
                set_label: "clear"
            }
        },

        filter_entry = gtk4::Entry {
            set_css_classes: &["google-translate-language-select-filter-entry"],
            set_placeholder_text: Some("Filter languages..."),
            set_hexpand: true,
            connect_changed: {
                let pages_stack = pages_stack.clone();
                let page_boxes = page_boxes.clone();
                let page_label = page_label.clone();
                let reveal_type = reveal_type.clone();
                let current_page = current_page.clone();
                let filter_clear_button = filter_clear_button.clone();

                move |entry| {
                    let filter_text = entry.text().to_string();
                    let filtered_boxes = get_page_boxes(
                        reveal_type.clone(),
                        if filter_text.is_empty() { None } else { Some(&filter_text) }
                    );

                    filter_clear_button.set_visible(!filter_text.is_empty());

                    page_boxes.lock().unwrap().clear();
                    page_boxes.lock().unwrap().extend(filtered_boxes);
                    
                    *current_page.lock().unwrap() = 0;
                    pages_stack.remove_all();
                    for (i, page_box) in page_boxes.lock().unwrap().iter_mut().enumerate() {
                        page_box.set_css_classes(&["google-translate-language-select-page"]);
                        pages_stack.add_named(page_box, Some(&format!("page_{}", i)));
                    }

                    pages_stack.set_visible_child_name("page_0");
                    pages_stack.set_visible_child_name(&format!("page_{}", *current_page.lock().unwrap()));

                    page_label.set_label(&format!(
                        "Page {} of {}",
                        *current_page.lock().unwrap() + 1,
                        page_boxes.lock().unwrap().len()
                    ));
                }
            }
        },

        widget = gtk4::Box {
            set_css_classes: &["google-translate-language-select-box"],
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 12,
            set_hexpand: true,

            gtk4::Label {
                set_css_classes: &["google-translate-language-select-label"],
                set_label: if reveal_type == LanguageSelectReveal::Source {
                    "Source Language"
                } else {
                    "Target Language"
                },
            },

            gtk4::Box {
                set_css_classes: &["google-translate-language-select-filter-box"],
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 6,

                append: &filter_entry,
                append: &filter_clear_button
            },

            libadwaita::Clamp {
                set_width_request: 400,
                set_maximum_size: 400,
                set_child: Some(&pages_stack),
                set_unit: libadwaita::LengthUnit::Px
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 6,
                set_homogeneous: true,

                gtk4::Button {
                    set_css_classes: &["google-translate-language-select-nav-button"],
                    set_label: "Previous",
                    connect_clicked: {
                        let pages_stack = pages_stack.clone();
                        let page_label = page_label.clone();
                        let page_boxes = page_boxes.clone();
                        let current_page = current_page.clone();

                        move |_| {
                            if *current_page.lock().unwrap() > 0 {
                                *current_page.lock().unwrap() -= 1;
                                pages_stack.set_visible_child_name(&format!("page_{}", current_page.lock().unwrap()));
                                page_label.set_label(&format!(
                                    "Page {} of {}",
                                    *current_page.lock().unwrap() + 1,
                                    page_boxes.lock().unwrap().len()
                                ));
                            }
                        }
                    }
                },

                append: &page_label,

                gtk4::Button {
                    set_css_classes: &["google-translate-language-select-nav-button"],
                    set_label: "Next",
                    connect_clicked: {
                        let pages_stack = pages_stack.clone();
                        let page_label = page_label.clone();
                        let page_boxes = page_boxes.clone();
                        let current_page = current_page.clone();

                        move |_| {
                            if *current_page.lock().unwrap() < page_boxes.lock().unwrap().len() - 1 {
                                *current_page.lock().unwrap() += 1;
                                pages_stack.set_visible_child_name(&format!("page_{}", current_page.lock().unwrap()));
                                page_label.set_label(&format!(
                                    "Page {} of {}",
                                    *current_page.lock().unwrap() + 1,
                                    page_boxes.lock().unwrap().len()
                                ));
                            }
                        }
                    }
                }
            }
        }
    };

    for (i, page_box) in page_boxes.lock().unwrap().iter_mut().enumerate() {
        page_box.set_css_classes(&["google-translate-language-select-page"]);
        pages_stack.add_named(page_box, Some(&format!("page_{}", i)));
    }
    
    pages_stack.set_visible_child_name("page_0");

    // Start our event receiver task
    let receiver = subscribe_to_ui_events();
    gtk4::glib::spawn_future_local({
        let pages_stack = pages_stack.clone();
        let page_boxes = page_boxes.clone();
        let current_page = current_page.clone();
        let filter_entry = filter_entry.clone();

        async move {
            while let Ok(event) = receiver.recv().await {
                if let UiEvent::LanguageSelectRevealChanged(reveal) = event {
                    if reveal != reveal_type {
                        continue;
                    }

                    *current_page.lock().unwrap() = 0;
                    pages_stack.set_visible_child_name("page_0");
                    pages_stack.remove_all();

                    for (i, page_box) in page_boxes.lock().unwrap().iter_mut().enumerate() {
                        page_box.set_css_classes(&["google-translate-language-select-page"]);
                        pages_stack.add_named(page_box, Some(&format!("page_{}", i)));
                    }

                    page_label.set_label(&format!(
                        "Page {} of {}",
                        *current_page.lock().unwrap() + 1,
                        page_boxes.lock().unwrap().len()
                    ));

                    filter_entry.set_text("");
                    filter_entry.grab_focus();
                }
            }
        }
    });

    widget
}