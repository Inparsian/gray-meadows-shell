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

enum UiEvent {
    TranslationStarted,
    TranslationFinished(Result<GoogleTranslateResult, String>),
    SourceLanguageChanged(Option<Language>),
    TargetLanguageChanged(Option<Language>),
}

fn is_working() -> bool {
    WORKING.try_lock().map(|w| *w).unwrap_or(true)
}

fn change_source_language(lang: Option<Language>, tx: &async_channel::Sender<UiEvent>) {
    let mut source_lang = SOURCE_LANG.lock().unwrap();
    *source_lang = lang.clone();

    tokio::spawn({
        let lang = lang.clone();
        let tx = tx.clone();

        async move {
            tx.send(UiEvent::SourceLanguageChanged(lang)).await.ok();
        }
    });
}

fn change_target_language(lang: Option<Language>, tx: &async_channel::Sender<UiEvent>) {
    let mut target_lang = TARGET_LANG.lock().unwrap();
    *target_lang = lang.clone();
    
    tokio::spawn({
        let lang = lang.clone();
        let tx = tx.clone();

        async move {
            tx.send(UiEvent::TargetLanguageChanged(lang)).await.ok();
        }
    });
}

async fn translate_future(
    text: String,
    source_lang: Option<Language>,
    target_lang: Option<Language>,
    autocorrect: bool,
    sender: async_channel::Sender<UiEvent>
) {
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
    let from_to_button_transition_provider = gtk4::CssProvider::new();

    let (tx, rx) = async_channel::bounded::<UiEvent>(1);

    relm4_macros::view! {
        source_lang_button = gtk4::Button {
            set_label: SOURCE_LANG.lock().unwrap().as_ref().map_or("Source...", |l| &l.name),
            set_css_classes: &["google-translate-language-select-button", "source-lang"],
            set_hexpand: true,
            connect_clicked: move |_| println!("Source language selection clicked")
        },

        target_lang_button = gtk4::Button {
            set_label: TARGET_LANG.lock().unwrap().as_ref().map_or("Target...", |l| &l.name),
            set_css_classes: &["google-translate-language-select-button", "target-lang"],
            set_hexpand: true,
            connect_clicked: move |_| println!("Target language selection clicked")
        },

        language_select_buttons = gtk4::Box {
            set_hexpand: true,
            
            append: &source_lang_button,
            gtk4::Label {
                set_css_classes: &["google-translate-arrow"],
                set_text: "→",
                set_hexpand: true
            },
            append: &target_lang_button
        },

        input_text_view = gtk4::TextView {
            set_wrap_mode: gtk4::WrapMode::WordChar,
            set_vexpand: true,
            set_hexpand: true,
            set_css_classes: &["google-translate-text-view"],
            set_buffer: Some(&input_buffer)
        },
        
        output_text_view = gtk4::TextView {
            set_wrap_mode: gtk4::WrapMode::WordChar,
            set_vexpand: true,
            set_hexpand: true,
            set_css_classes: &["google-translate-text-view"],
            set_buffer: Some(&output_buffer),
            set_editable: false
        },

        widget = gtk4::Box {
            set_css_classes: &["GoogleTranslate"],
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 8,
            set_hexpand: true,
            set_vexpand: true,

            append: &language_select_buttons,

            gtk4::ScrolledWindow {
                set_child: Some(&input_text_view)
            },

            gtk4::ScrolledWindow {
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
                        let source_lang_button = source_lang_button.clone();
                        let target_lang_button = target_lang_button.clone();
                        let language_select_buttons = language_select_buttons.clone();
                        let from_to_button_transition_provider = from_to_button_transition_provider.clone();
                        let tx = tx.clone();

                        move |_| if !is_working() {
                            let input_text = input_buffer.text(&input_buffer.start_iter(), &input_buffer.end_iter(), false).to_string();
                            let output_text = output_buffer.text(&output_buffer.start_iter(), &output_buffer.end_iter(), false).to_string();
                            let source_lang_cloned = SOURCE_LANG.lock().unwrap().clone();
                            let target_lang_cloned = TARGET_LANG.lock().unwrap().clone();

                            change_source_language(target_lang_cloned, &tx);
                            change_target_language(source_lang_cloned, &tx);

                            input_buffer.set_text(&output_text);
                            output_buffer.set_text(&input_text);

                            // Button swap animation
                            let source_lang_allocation = source_lang_button.allocation();
                            let target_lang_allocation = target_lang_button.allocation();
                            let buttons_box_allocation = language_select_buttons.allocation();
                            
                            from_to_button_transition_provider.load_from_data(&format!("
                                .google-translate-language-select-button.source-lang {{
                                    transition: none;
                                    margin-left: {}px;
                                    margin-right: -{}px;
                                }}

                                .google-translate-language-select-button.target-lang {{
                                    transition: none;
                                    margin-left: -{}px;
                                    margin-right: {}px;
                                }}",
                                buttons_box_allocation.width() - source_lang_allocation.width() + 1,
                                buttons_box_allocation.width() - source_lang_allocation.width() + 1,
                                buttons_box_allocation.width() - target_lang_allocation.width() + 1,
                                buttons_box_allocation.width() - target_lang_allocation.width() + 1
                            ));

                            gtk4::glib::timeout_add_local_once(Duration::from_millis(10), {
                                let from_to_button_transition_provider = from_to_button_transition_provider.clone();
                                move || from_to_button_transition_provider.load_from_data("
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
        }
    };

    source_lang_button.style_context().add_provider(
        &from_to_button_transition_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION
    );

    target_lang_button.style_context().add_provider(
        &from_to_button_transition_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION
    );

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
                        let source_lang = SOURCE_LANG.lock().unwrap().clone();
                        let target_lang = TARGET_LANG.lock().unwrap().clone();
                        let tx = tx.clone();

                        move || tokio::runtime::Runtime::new().unwrap().block_on(translate_future(
                            text,
                            source_lang,
                            target_lang,
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
        let source_lang_button = source_lang_button.clone();
        let target_lang_button = target_lang_button.clone();

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

                                source_lang_button.set_label(&format!("Auto ({})", res.from.language.name));
                            }
                        } else {
                            output_buffer.set_text(&format!("Translation failed:\n{}", result.unwrap_err()));
                        }

                        input_text_view.set_editable(true);
                    },

                    UiEvent::SourceLanguageChanged(lang) => {
                        source_lang_button.set_label(lang.as_ref().map_or("Source...", |l| &l.name));
                    },

                    UiEvent::TargetLanguageChanged(lang) => {
                        target_lang_button.set_label(lang.as_ref().map_or("Target...", |l| &l.name));
                    }
                }
            }
        }
    });

    widget
}