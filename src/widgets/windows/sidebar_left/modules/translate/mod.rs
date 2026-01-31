mod lang_buttons;
mod lang_select;

use std::sync::{RwLock, LazyLock};
use std::time::Duration;
use async_broadcast::Receiver;
use gtk4::prelude::*;

use crate::services::g_translate::languages::{self, Language};
use crate::services::g_translate::result::GoogleTranslateResult;
use crate::services::g_translate::translate;
use crate::utils::broadcast::BroadcastChannel;
use crate::utils::timeout::Timeout;
use self::lang_buttons::{LanguageButtons, LanguageButtonsKind};
use self::lang_select::LanguageSelectView;

static WORKING: LazyLock<RwLock<bool>> = LazyLock::new(|| RwLock::new(false));
static SOURCE_LANG: LazyLock<RwLock<Option<Language>>> = LazyLock::new(|| RwLock::new(languages::get_by_code("en")));
static TARGET_LANG: LazyLock<RwLock<Option<Language>>> = LazyLock::new(|| RwLock::new(languages::get_by_code("es")));
static AUTO_DETECTED_LANG: LazyLock<RwLock<Option<Language>>> = LazyLock::new(|| RwLock::new(None));
static REVEAL: LazyLock<RwLock<LanguageSelectReveal>> = LazyLock::new(|| RwLock::new(LanguageSelectReveal::None));
static UI_EVENT_CHANNEL: LazyLock<BroadcastChannel<UiEvent>> = LazyLock::new(|| BroadcastChannel::new(10));

#[derive(Debug, Clone, PartialEq, Eq, glib::Downgrade)]
pub enum LanguageSelectReveal {
    Source,
    Target,
    None
}

#[derive(Debug, Clone)]
pub enum UiEvent {
    TranslationStarted,
    TranslationFinished(Result<GoogleTranslateResult, String>),
    SourceLanguageChanged(Option<Language>),
    TargetLanguageChanged(Option<Language>),
    LanguageSelectRevealChanged(LanguageSelectReveal)
}

fn is_working() -> bool {
    WORKING.try_read().map(|w| *w).unwrap_or(true)
}

pub fn subscribe_to_ui_events() -> Receiver<UiEvent> {
    UI_EVENT_CHANNEL.subscribe()
}

pub fn send_ui_event(event: &UiEvent) {
    tokio::spawn(clone!(
        #[strong] event,
        async move {
            UI_EVENT_CHANNEL.send(event).await;
        }
    ));
}

pub fn set_source_language(lang: Option<Language>) {
    let mut source_lang = SOURCE_LANG.write().unwrap();
    *source_lang = lang.clone();

    send_ui_event(&UiEvent::SourceLanguageChanged(lang));
}

pub fn set_target_language(lang: Option<Language>) {
    let mut target_lang = TARGET_LANG.write().unwrap();
    *target_lang = lang.clone();
    
    send_ui_event(&UiEvent::TargetLanguageChanged(lang));
}

async fn translate_future(text: String, autocorrect: bool) {
    let source_lang = SOURCE_LANG.read().unwrap().clone();
    let target_lang = TARGET_LANG.read().unwrap().clone();

    if let (Some(source_lang), Some(target_lang)) = (source_lang, target_lang) {
        if !*WORKING.read().unwrap() {
            *WORKING.write().unwrap() = true;
            send_ui_event(&UiEvent::TranslationStarted);

            let translation_result = translate(&text, source_lang, target_lang, autocorrect)
                .await
                .map_err(|e| e.to_string());

            send_ui_event(&UiEvent::TranslationFinished(translation_result));

            // Keep a hold of the working state for a while longer to prevent
            // an infinite translation loop due to buffer change signals.
            tokio::time::sleep(Duration::from_millis(10)).await;
            *WORKING.write().unwrap() = false;
        } else {
            warn!("Translation already in progress");
        }
    } else {
        warn!("Invalid language selection for translation");
    }
}

pub fn new() -> gtk4::Box {
    let timeout = Timeout::default();
    let input_buffer = gtk4::TextBuffer::new(None);
    let output_buffer = gtk4::TextBuffer::new(None);

    let source_language_buttons = LanguageButtons::new(LanguageButtonsKind::Source);
    let target_language_buttons = LanguageButtons::new(LanguageButtonsKind::Target);
    let source_select_view = LanguageSelectView::new(LanguageSelectReveal::Source);
    let target_select_view = LanguageSelectView::new(LanguageSelectReveal::Target);

    view! {
        input_text_view = gtk4::TextView {
            set_wrap_mode: gtk4::WrapMode::WordChar,
            set_hexpand: true,
            set_css_classes: &["google-translate-text-view"],
            set_buffer: Some(&input_buffer)
        },
        
        output_text_view = gtk4::TextView {
            set_wrap_mode: gtk4::WrapMode::WordChar,
            set_hexpand: true,
            set_css_classes: &["google-translate-text-view"],
            set_buffer: Some(&output_buffer),
            set_editable: false
        },

        main_ui = gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 0,
            set_hexpand: true,
            
            append: &source_language_buttons.container,

            gtk4::ScrolledWindow {
                set_vexpand: true,
                set_child: Some(&input_text_view)
            },
            
            append: &target_language_buttons.container,

            gtk4::ScrolledWindow {
                set_vexpand: true,
                set_child: Some(&output_text_view)
            },

            gtk4::Box {
                set_homogeneous: true,
                set_spacing: 16,

                gtk4::Button {
                    set_css_classes: &["google-translate-button"],
                    connect_clicked: clone!(
                        #[weak] input_buffer,
                        #[weak] output_buffer,
                        move |_| if !is_working() {
                            input_buffer.set_text("");
                            output_buffer.set_text("");
                        }
                    ),

                    gtk4::Box {
                        set_spacing: 4,
                        set_halign: gtk4::Align::Center,

                        gtk4::Label {
                            set_css_classes: &["material-icons"],
                            set_text: "delete"
                        },

                        gtk4::Label {
                            set_text: "Clear"
                        }
                    }
                },

                gtk4::Button {
                    set_css_classes: &["google-translate-button"],
                    connect_clicked: clone!(
                        #[weak] input_buffer,
                        #[weak] output_buffer,
                        move |_| if !is_working() {
                            let source_lang = SOURCE_LANG.read().unwrap().clone();
                            let target_lang = TARGET_LANG.read().unwrap().clone();
                            
                            if source_lang.as_ref().is_none_or(|lang| !lang.is_auto())
                                && target_lang.as_ref().is_none_or(|lang| !lang.is_auto())
                            {
                                let input_text = input_buffer.text(&input_buffer.start_iter(), &input_buffer.end_iter(), false).to_string();
                                let output_text = output_buffer.text(&output_buffer.start_iter(), &output_buffer.end_iter(), false).to_string();
    
                                set_source_language(target_lang);
                                set_target_language(source_lang);
    
                                input_buffer.set_text(&output_text);
                                output_buffer.set_text(&input_text);
                            }
                        }
                    ),
                            
                    gtk4::Box {
                        set_spacing: 4,
                        set_halign: gtk4::Align::Center,

                        gtk4::Label {
                            set_css_classes: &["material-icons"],
                            set_text: "swap_horiz"
                        },

                        gtk4::Label {
                            set_text: "Swap"
                        }
                    }
                }
            }
        },

        select_ui_stack = gtk4::Stack {
            set_hexpand: true,
            set_transition_type: gtk4::StackTransitionType::SlideLeftRight,
            set_transition_duration: 0,

            add_named: (source_select_view.get_widget(), Some("source")),
            add_named: (target_select_view.get_widget(), Some("target"))
        },

        select_ui = gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 8,
            set_hexpand: true,
            append: &select_ui_stack
        },

        ui_stack = gtk4::Stack {
            set_hexpand: true,
            set_transition_type: gtk4::StackTransitionType::SlideLeftRight,
            set_transition_duration: 250,

            add_named: (&main_ui, Some("main")),
            add_named: (&select_ui, Some("select")),
            
            set_visible_child_name: "main"
        },

        widget = gtk4::Box {
            set_css_classes: &["GoogleTranslate"],
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 8,
            set_hexpand: true,

            append: &ui_stack,
        }
    };

    input_buffer.connect_changed(move |buffer| {
        if is_working() {
            return;
        }

        let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false).to_string();
        if !text.is_empty() {
            timeout.set(Duration::from_millis(500), move || {
                tokio::spawn(translate_future(text, false));
            });
        }
    });

    // Start our event receiver task
    glib::spawn_future_local(async move {
        let mut receiver = subscribe_to_ui_events();
        while let Ok(event) = receiver.recv().await {
            match event {
                UiEvent::TranslationStarted => {
                    output_buffer.set_text("Translating...");
                    input_text_view.set_editable(false);
                },

                UiEvent::TranslationFinished(result) => {
                    match result {
                        Ok(res) => {
                            output_buffer.set_text(&res.to.text);

                            // Set the auto-detected language if applicable
                            if SOURCE_LANG.read().unwrap().as_ref().unwrap().is_auto() {
                                *AUTO_DETECTED_LANG.write().unwrap() = Some(res.from.language.clone());

                                source_language_buttons.set_auto_language(&res.from.language.name);
                            }
                        },

                        Err(err_msg) => {
                            output_buffer.set_text(&format!("Translation failed:\n{}", err_msg));
                        }
                    }

                    input_text_view.set_editable(true);
                },

                UiEvent::SourceLanguageChanged(lang) => if let Some(lang) = lang {
                    source_language_buttons.push_language(lang);
                },

                UiEvent::TargetLanguageChanged(lang) => if let Some(lang) = lang {
                    target_language_buttons.push_language(lang);
                },

                UiEvent::LanguageSelectRevealChanged(reveal) => {
                    let was_already_open = reveal == *REVEAL.read().unwrap();

                    ui_stack.set_visible_child_name(if was_already_open || reveal == LanguageSelectReveal::None {
                        "main"
                    } else {
                        "select"
                    });

                    if matches!(&reveal, LanguageSelectReveal::Source | LanguageSelectReveal::Target) {
                        select_ui_stack.set_visible_child_name(match reveal {
                            LanguageSelectReveal::Target => "target",
                            LanguageSelectReveal::Source => "source",
                            _ => unreachable!()
                        });
                    }

                    *REVEAL.write().unwrap() = if was_already_open {
                        LanguageSelectReveal::None
                    } else {
                        reveal
                    };
                }
            }
        }
    });

    widget
}