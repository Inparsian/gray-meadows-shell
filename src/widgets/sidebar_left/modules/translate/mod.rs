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

async fn translate_future(
    from_language: Option<Language>,
    to_language: Option<Language>,
    text: String,
    autocorrect: bool,
    sender: async_channel::Sender<TranslationEvent>
) {
    if let (Some(from_lang), Some(to_lang)) = (from_language, to_language) {
        let can_proceed = {
            if let Ok(mut working) = WORKING.lock() {
                *working = true;
                true
            } else {
                false
            }
        };

        if can_proceed {
            sender.send(TranslationEvent::TranslationStarted).await.ok();

            let translation_result = translate(
                &text,
                from_lang,
                to_lang,
                autocorrect
            ).await;

            sender.send(TranslationEvent::TranslationFinished(
                translation_result.map_err(|e| e.to_string())
            )).await.ok();

            // Keep a hold of the working state for a while longer to prevent
            // an infinite translation loop due to buffer change signals.
            std::thread::sleep(Duration::from_millis(10));
            
            if let Ok(mut working) = WORKING.lock() {
                *working = false;
            }
        } else {
            eprintln!("Translation already in progress");
        }
    } else {
        eprintln!("Invalid language selection for translation");
    }
}

pub fn new() -> gtk4::Box {
    // TODO: Make these mutable by the user
    let from_language = language::get_by_name("English");
    let to_language = language::get_by_name("Spanish");

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
            }
        }
    };

    input_buffer.connect_changed({
        let from_language = from_language.clone();
        let to_language = to_language.clone();
        let tx = tx.clone();

        move |buffer| {
            if let Ok(working) = WORKING.try_lock() {
                if *working {
                    return;
                }
            } else {
                return;
            }

            let text = buffer.text(
                &buffer.start_iter(), 
                &buffer.end_iter(), 
                false
            ).to_string();

            let mut worker_timeout = WORKER_TIMEOUT.lock().unwrap();
            if let Some(source_id) = worker_timeout.take() {
                source_id.remove();
            }

            if !text.is_empty() {
                *worker_timeout = Some(gtk4::glib::timeout_add_local_once(Duration::from_millis(500), {
                    let from_language = from_language.clone();
                    let to_language = to_language.clone();
                    let tx = tx.clone();

                    move || {
                        if let Ok(mut worker_timeout) = WORKER_TIMEOUT.lock() {
                            *worker_timeout = None;
                        }

                        std::thread::spawn({
                            let from_language = from_language.clone();
                            let to_language = to_language.clone();
                            let text = text.clone();
                            let tx = tx.clone();

                            move || tokio::runtime::Runtime::new().unwrap().block_on(translate_future(
                                from_language,
                                to_language,
                                text,
                                false,
                                tx
                            ))
                        });
                    }
                }));
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