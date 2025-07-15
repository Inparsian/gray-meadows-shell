use std::{sync::Mutex, time::Duration};
use once_cell::sync::Lazy;
use gtk4::prelude::*;

use crate::singletons::g_translate::{language::{self, Language}, result::GoogleTranslateResult, translate};

static WORKER_TIMEOUT: Lazy<Mutex<Option<gtk4::glib::SourceId>>> = Lazy::new(|| Mutex::new(None));
static WORKING: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

enum TranslationEvent {
    TranslationStarted,
    TranslationFinished(Result<GoogleTranslateResult, String>),
}

fn is_working() -> bool {
    WORKING.try_lock().map(|w| *w).unwrap_or(true)
}

async fn translate_future(
    text: String,
    source_lang: Option<Language>,
    target_lang: Option<Language>,
    autocorrect: bool,
    sender: async_channel::Sender<TranslationEvent>
) {
    if let (Some(source_lang), Some(target_lang)) = (source_lang, target_lang) {
        if WORKING.lock().map(|mut w| *w = true).is_ok() {
            sender.send(TranslationEvent::TranslationStarted).await.ok();

            let translation_result = translate(&text, source_lang, target_lang, autocorrect)
                .await
                .map_err(|e| e.to_string());

            sender.send(TranslationEvent::TranslationFinished(translation_result)).await.ok();

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
    // TODO: Make these mutable by the user
    let source_lang = language::get_by_name("English");
    let target_lang = language::get_by_name("Spanish");

    let input_buffer = gtk4::TextBuffer::new(None);
    let output_buffer = gtk4::TextBuffer::new(None);

    let (tx, rx) = async_channel::bounded::<TranslationEvent>(1);

    relm4_macros::view! {
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
                }
            }
        }
    };

    input_buffer.connect_changed({
        let source_lang = source_lang.clone();
        let target_lang = target_lang.clone();
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
                let source_lang = source_lang.clone();
                let target_lang = target_lang.clone();
                let tx = tx.clone();

                let timeout = gtk4::glib::timeout_add_local_once(Duration::from_millis(500), move || {
                    WORKER_TIMEOUT.lock().unwrap().take();

                    std::thread::spawn({
                        let source_lang = source_lang.clone();
                        let target_lang = target_lang.clone();
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

        async move {
            while let Ok(event) = rx.recv().await {
                match event {
                    TranslationEvent::TranslationStarted => {
                        output_buffer.set_text("Translating...");
                        input_text_view.set_editable(false);
                    },

                    TranslationEvent::TranslationFinished(result) => {
                        if let Ok(res) = result {
                            output_buffer.set_text(&res.to.text);
                        } else {
                            output_buffer.set_text(&format!("Translation failed:\n{}", result.unwrap_err()));
                        }

                        input_text_view.set_editable(true);
                    }
                }
            }
        }
    });

    widget
}