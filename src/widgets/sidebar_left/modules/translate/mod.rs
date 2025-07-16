mod lang_buttons;
mod lang_select;

use std::{sync::Mutex, time::Duration};
use once_cell::sync::Lazy;
use gtk4::prelude::*;

use crate::singletons::g_translate::{
    language::{self, Language, AUTO_LANG},
    result::GoogleTranslateResult, translate
};

static WORKER_TIMEOUT: Lazy<Mutex<Option<gtk4::glib::SourceId>>> = Lazy::new(|| Mutex::new(None));
static WORKING: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

static SOURCE_LANG: Lazy<Mutex<Option<Language>>> = Lazy::new(|| Mutex::new(language::get_by_name("English")));
static TARGET_LANG: Lazy<Mutex<Option<Language>>> = Lazy::new(|| Mutex::new(language::get_by_name("Spanish")));
static AUTO_DETECTED_LANG: Lazy<Mutex<Option<Language>>> = Lazy::new(|| Mutex::new(None));
static REVEAL: Lazy<Mutex<LanguageSelectReveal>> = Lazy::new(|| Mutex::new(LanguageSelectReveal::None));

#[derive(Debug, Clone, PartialEq)]
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
    WORKING.try_lock().map(|w| *w).unwrap_or(true)
}

pub fn send_ui_event(event: UiEvent, sender: &async_channel::Sender<UiEvent>) {
    tokio::spawn({
        let event = event.clone();
        let tx = sender.clone();

        async move {
            tx.send(event).await.ok();
        }
    });
}

pub fn set_source_language(lang: Option<Language>, tx: &async_channel::Sender<UiEvent>) {
    let mut source_lang = SOURCE_LANG.lock().unwrap();
    *source_lang = lang.clone();

    send_ui_event(UiEvent::SourceLanguageChanged(lang), tx);
}

pub fn set_target_language(lang: Option<Language>, tx: &async_channel::Sender<UiEvent>) {
    let mut target_lang = TARGET_LANG.lock().unwrap();
    *target_lang = lang.clone();
    
    send_ui_event(UiEvent::TargetLanguageChanged(lang), tx);
}

async fn translate_future(
    text: String,
    autocorrect: bool,
    sender: async_channel::Sender<UiEvent>
) {
    let source_lang = SOURCE_LANG.lock().unwrap().clone();
    let target_lang = TARGET_LANG.lock().unwrap().clone();

    if let (Some(source_lang), Some(target_lang)) = (source_lang, target_lang) {
        if WORKING.lock().map(|mut w| *w = true).is_ok() {
            sender.send(UiEvent::TranslationStarted).await.ok();

            let translation_result = translate(&text, source_lang, target_lang, autocorrect)
                .await
                .map_err(|e| e.to_string());

            sender.send(UiEvent::TranslationFinished(translation_result)).await.ok();

            // Keep a hold of the working state for a while longer to prevent
            // an infinite translation loop due to buffer change signals.
            std::thread::sleep(Duration::from_millis(10));
            let _ = WORKING.lock().map(|mut w| *w = false);
        } else {
            eprintln!("Translation already in progress");
        }
    } else {
        eprintln!("Invalid language selection for translation");
    }
}

pub fn new() -> gtk4::Box {
    let input_buffer = gtk4::TextBuffer::new(None);
    let output_buffer = gtk4::TextBuffer::new(None);

    let (tx, rx) = async_channel::bounded::<UiEvent>(1);

    let language_buttons = lang_buttons::LanguageButtons::new(tx.clone());

    relm4_macros::view! {
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
            set_spacing: 8,
            set_hexpand: true,

            gtk4::ScrolledWindow {
                set_height_request: 400,
                set_child: Some(&input_text_view)
            },

            gtk4::ScrolledWindow {
                set_height_request: 400,
                set_child: Some(&output_text_view)
            },

            gtk4::Box {
                set_homogeneous: true,
                set_spacing: 16,

                gtk4::Button {
                    set_css_classes: &["google-translate-button"],
                    connect_clicked: {
                        let input_buffer = input_buffer.clone();
                        let output_buffer = output_buffer.clone();

                        move |_| if !is_working() {
                            input_buffer.set_text("");
                            output_buffer.set_text("");
                        }
                    },

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
                    connect_clicked: {
                        let input_buffer = input_buffer.clone();
                        let output_buffer = output_buffer.clone();
                        let language_buttons = language_buttons.clone();
                        let tx = tx.clone();

                        move |_| if !is_working() {
                            let input_text = input_buffer.text(&input_buffer.start_iter(), &input_buffer.end_iter(), false).to_string();
                            let output_text = output_buffer.text(&output_buffer.start_iter(), &output_buffer.end_iter(), false).to_string();
                            let source_lang_cloned = SOURCE_LANG.lock().unwrap().clone();
                            let target_lang_cloned = TARGET_LANG.lock().unwrap().clone();

                            set_source_language(target_lang_cloned, &tx);
                            set_target_language(source_lang_cloned, &tx);
                            language_buttons.swap_animation();

                            input_buffer.set_text(&output_text);
                            output_buffer.set_text(&input_text);
                        }
                    },
                            
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

        main_ui_revealer = gtk4::Revealer {
            set_reveal_child: REVEAL.lock().unwrap().clone() == LanguageSelectReveal::None,
            set_transition_type: gtk4::RevealerTransitionType::SlideDown,
            set_transition_duration: 250,
            set_child: Some(&main_ui)
        },

        select_ui_stack = gtk4::Stack {
            set_hexpand: true,
            set_transition_type: gtk4::StackTransitionType::SlideLeftRight,
            set_transition_duration: 250,

            add_named: (&lang_select::new(LanguageSelectReveal::Source, tx.clone()), Some("source")),
            add_named: (&lang_select::new(LanguageSelectReveal::Target, tx.clone()), Some("target"))
        },

        select_ui = gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 8,
            set_hexpand: true,
            append: &select_ui_stack
        },

        select_ui_revealer = gtk4::Revealer {
            set_reveal_child: REVEAL.lock().unwrap().clone() != LanguageSelectReveal::None,
            set_transition_type: gtk4::RevealerTransitionType::SlideDown,
            set_transition_duration: 250,
            set_child: Some(&select_ui)
        },

        widget = gtk4::Box {
            set_css_classes: &["GoogleTranslate"],
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 8,
            set_hexpand: true,

            append: &language_buttons.container,
            append: &select_ui_revealer,
            append: &main_ui_revealer,
        }
    };

    input_buffer.connect_changed({
        let tx = tx.clone();

        move |buffer| {
            if is_working() {
                return;
            }

            if let Some(source_id) = WORKER_TIMEOUT.lock().unwrap().take() {
                source_id.remove();
            }

            let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false).to_string();
            if !text.is_empty() {
                let tx = tx.clone();

                let timeout = gtk4::glib::timeout_add_local_once(Duration::from_millis(500), move || {
                    WORKER_TIMEOUT.lock().unwrap().take();

                    std::thread::spawn({
                        let tx = tx.clone();

                        move || tokio::runtime::Runtime::new().unwrap().block_on(translate_future(
                            text,
                            false,
                            tx
                        ))
                    });
                });

                *WORKER_TIMEOUT.lock().unwrap() = Some(timeout);
            }
        }
    });

    // Start our receiver task
    gtk4::glib::spawn_future_local({
        let output_buffer = output_buffer.clone();
        let input_text_view = input_text_view.clone();
        let language_buttons = language_buttons.clone();
        let main_ui_revealer = main_ui_revealer.clone();
        let select_ui_revealer = select_ui_revealer.clone();

        async move {
            while let Ok(event) = rx.recv().await {
                match event {
                    UiEvent::TranslationStarted => {
                        output_buffer.set_text("Translating...");
                        input_text_view.set_editable(false);
                    },

                    UiEvent::TranslationFinished(result) => {
                        if let Ok(res) = result {
                            output_buffer.set_text(&res.to.text);

                            // Set the auto-detected language if applicable
                            if SOURCE_LANG.lock().unwrap().as_ref().unwrap() == &*AUTO_LANG {
                                let mut auto_detected_lang = AUTO_DETECTED_LANG.lock().unwrap();

                                *auto_detected_lang = Some(res.from.language.clone());

                                language_buttons.set_source_label(&format!("Auto ({})", res.from.language.name));
                            }
                        } else {
                            output_buffer.set_text(&format!("Translation failed:\n{}", result.unwrap_err()));
                        }

                        input_text_view.set_editable(true);
                    },

                    UiEvent::SourceLanguageChanged(lang) => {
                        language_buttons.set_source_label(lang.as_ref().map_or("Source...", |l| &l.name));
                    },

                    UiEvent::TargetLanguageChanged(lang) => {
                        language_buttons.set_target_label(lang.as_ref().map_or("Target...", |l| &l.name));
                    },

                    UiEvent::LanguageSelectRevealChanged(reveal) => {
                        let was_already_open = reveal == *REVEAL.lock().unwrap();

                        main_ui_revealer.set_reveal_child(was_already_open || reveal == LanguageSelectReveal::None);
                        select_ui_revealer.set_reveal_child(!was_already_open && reveal != LanguageSelectReveal::None);
                        select_ui_stack.set_visible_child_name(match reveal {
                            LanguageSelectReveal::Source => "source",
                            LanguageSelectReveal::Target => "target",
                            LanguageSelectReveal::None => "source" // Default to source when hidden
                        });

                        *REVEAL.lock().unwrap() = if was_already_open {
                            LanguageSelectReveal::None
                        } else {
                            reveal
                        };
                    }
                }
            }
        }
    });

    widget
}