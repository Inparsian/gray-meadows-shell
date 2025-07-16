use std::time::Duration;

use gtk4::prelude::*;

use crate::widgets::sidebar_left::modules::translate::{send_ui_event, LanguageSelectReveal, UiEvent, SOURCE_LANG, TARGET_LANG};

#[derive(Debug, Clone)]
pub struct LanguageButtons {
    pub source_lang_button: gtk4::Button,
    pub target_lang_button: gtk4::Button,
    pub container: gtk4::Box,
    swap_transition_provider: gtk4::CssProvider,
}

impl LanguageButtons {
    pub fn new() -> Self {
        let swap_transition_provider = gtk4::CssProvider::new();

        relm4_macros::view! {
            source_lang_button = gtk4::Button {
                set_label: SOURCE_LANG.lock().unwrap().as_ref().map_or("Source...", |l| &l.name),
                set_css_classes: &["google-translate-language-select-button", "source-lang"],
                set_hexpand: true,
                connect_clicked: move |_| send_ui_event(UiEvent::LanguageSelectRevealChanged(LanguageSelectReveal::Source))
            },

            target_lang_button = gtk4::Button {
                set_label: TARGET_LANG.lock().unwrap().as_ref().map_or("Target...", |l| &l.name),
                set_css_classes: &["google-translate-language-select-button", "target-lang"],
                set_hexpand: true,
                connect_clicked: move |_| send_ui_event(UiEvent::LanguageSelectRevealChanged(LanguageSelectReveal::Target))
            },

            container = gtk4::Box {
                set_hexpand: true,

                append: &source_lang_button,
                gtk4::Label {
                    set_css_classes: &["google-translate-arrow"],
                    set_text: "→",
                    set_hexpand: true
                },
                append: &target_lang_button
            },
        };

        source_lang_button.style_context().add_provider(
            &swap_transition_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        target_lang_button.style_context().add_provider(
            &swap_transition_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        Self {
            source_lang_button,
            target_lang_button,
            container,
            swap_transition_provider,
        }
    }

    pub fn swap_animation(&self) {
        let source_lang_allocation = self.source_lang_button.allocation();
        let target_lang_allocation = self.target_lang_button.allocation();
        let buttons_box_allocation = self.container.allocation();
        let source_margin = buttons_box_allocation.width() - source_lang_allocation.width() + 1;
        let target_margin = buttons_box_allocation.width() - target_lang_allocation.width() + 1;

        self.swap_transition_provider.load_from_data(&format!("
            .google-translate-language-select-button.source-lang {{
                transition: none;
                margin-left: {source_margin}px;
                margin-right: -{source_margin}px;
            }}

            .google-translate-language-select-button.target-lang {{
                transition: none;
                margin-left: -{target_margin}px;
                margin-right: {target_margin}px;
            }}
        "));

        gtk4::glib::timeout_add_local_once(Duration::from_millis(10), {
            let swap_transition_provider = self.swap_transition_provider.clone();
            move || swap_transition_provider.load_from_data("
                .google-translate-language-select-button.source-lang,
                .google-translate-language-select-button.target-lang {
                    transition: margin-left 0.33s cubic-bezier(0.5, 0.8, 0, 1),
                                margin-right 0.33s cubic-bezier(0.5, 0.8, 0, 1);
                    margin-left: 0px;
                    margin-right: 0px;
                }
            ")
        });
    }

    pub fn set_source_label(&self, language: &str) {
        self.source_lang_button.set_label(language);
    }

    pub fn set_target_label(&self, language: &str) {
        self.target_lang_button.set_label(language);
    }
}