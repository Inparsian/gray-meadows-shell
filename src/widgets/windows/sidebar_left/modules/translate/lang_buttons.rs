use std::rc::Rc;
use std::cell::RefCell;
use gtk4::prelude::*;

use crate::services::g_translate::languages::{self, Language};
use crate::sql::wrappers::translate::LANGUAGES_LIMIT;
use crate::sql::wrappers::{translate, state};
use super::{send_ui_event, LanguageSelectReveal, UiEvent};

#[derive(Debug, Clone, PartialEq, Eq, glib::Downgrade)]
pub enum LanguageButtonsKind {
    Source,
    Target,
}

#[derive(Debug, Clone, glib::Downgrade)]
pub struct LanguageButtons {
    pub kind: LanguageButtonsKind,
    pub buttons: Rc<RefCell<Vec<(Language, gtk4::Button)>>>,
    pub container: gtk4::Box,
    pub languages_box: gtk4::Box,
}

impl LanguageButtons {
    fn language_button(kind: &LanguageButtonsKind, language: Language) -> (Language, gtk4::Button) {
        let button = gtk4::Button::builder()
            .css_classes(["google-translate-language-button"])
            .label(&language.name)
            .build();

        button.connect_clicked(clone!(
            #[strong] kind,
            #[strong] language,
            move |_| {
                let language = Some(language.clone());
                if kind == LanguageButtonsKind::Source {
                    super::set_source_language(language);
                } else {
                    super::set_target_language(language);
                }
            }
        ));

        (language, button)
    }
    
    pub fn new(kind: LanguageButtonsKind) -> Self {
        let buttons = Rc::new(RefCell::new(Vec::new()));
        let container = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(0)
            .hexpand(true)
            .build();
        
        let languages_box_scrolled = gtk4::ScrolledWindow::builder()
            .css_classes(["google-translate-languages-scrolled-window"])
            .hscrollbar_policy(gtk4::PolicyType::Automatic)
            .vscrollbar_policy(gtk4::PolicyType::Never)
            .hexpand(true)
            .build();
        
        let languages_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(0)
            .hexpand(true)
            .build();
        
        languages_box_scrolled.set_child(Some(&languages_box));
        container.append(&languages_box_scrolled);
        
        if kind == LanguageButtonsKind::Source {
            languages_box.append(&Self::language_button(&kind, Language::auto()).1);
        }
        
        let select = gtk4::Button::builder()
            .halign(gtk4::Align::End)
            .css_classes(["google-translate-language-select-menu-button"])
            .label("chevron_right")
            .build();
        
        select.connect_clicked(clone!(
            #[strong] kind,
            move |_| {
                send_ui_event(&UiEvent::LanguageSelectRevealChanged(if kind == LanguageButtonsKind::Source {
                    LanguageSelectReveal::Source
                } else {
                    LanguageSelectReveal::Target
                }));
            }
        ));
        
        container.append(&select);
        
        glib::spawn_future_local(clone!(
            #[strong] kind,
            #[weak] buttons,
            #[weak] languages_box,
            async move {
                let lang = if kind == LanguageButtonsKind::Source {
                    state::get_source_language().await
                } else {
                    state::get_target_language().await
                };
                
                let list = if kind == LanguageButtonsKind::Source {
                    translate::get_last_source_languages().await
                } else {
                    translate::get_last_target_languages().await
                };
                
                let lang_buttons = match list {
                    Ok(list) => list.into_iter()
                        .map(|language| Self::language_button(&kind, language))
                        .rev()
                        .collect(),
                    
                    Err(err) => {
                        error!("Failed to get last three languages: {}", err);
                        Vec::new()
                    }
                };
                
                for button in &lang_buttons {
                    languages_box.append(&button.1);
                    buttons.borrow_mut().push(button.clone());
                }
                
                let language = match lang {
                    Ok(code) => languages::get_by_code(&code),
                    Err(err) => {
                        error!("Failed to get language from code: {}", err);
                        None
                    }
                };
                
                if kind == LanguageButtonsKind::Source {
                    super::set_source_language(language);
                } else {
                    super::set_target_language(language);
                }
            }
        ));
        
        Self {
            kind,
            buttons,
            container,
            languages_box,
        }
    }
    
    pub fn push_language(&self, language: Language) {
        let mut buttons = self.buttons.borrow_mut();
        
        if self.kind == LanguageButtonsKind::Source && language.is_auto() {
            for button in buttons.iter() {
                button.1.remove_css_class("selected");
            }
            
            if let Some(button) = self.languages_box.first_child()
                .and_then(|child| child.downcast::<gtk4::Button>().ok())
            {
                button.add_css_class("selected");
            }
            
            tokio::spawn(async {
                let auto = Language::auto();
                let _ = state::set_source_language(&auto.code).await;
            });
            return;
        } else if let Some(selected_button) = buttons.iter().find(|b| b.0 == language) {
            for button in buttons.iter() {
                button.1.remove_css_class("selected");
            }
            
            if self.kind == LanguageButtonsKind::Source
                && let Some(button) = self.languages_box.first_child()
                    .and_then(|child| child.downcast::<gtk4::Button>().ok())
            {
                button.remove_css_class("selected");
            }
            
            selected_button.1.add_css_class("selected");
            
            tokio::spawn(clone!(
                #[strong(rename_to = kind)] self.kind,
                #[strong(rename_to = code)] selected_button.0.code,
                async move {
                    if kind == LanguageButtonsKind::Source {
                        let _ = state::set_source_language(&code).await;
                    } else {
                        let _ = state::set_target_language(&code).await;
                    }
                }
            ));
            return;
        }
        
        let button = Self::language_button(&self.kind, language.clone());
        
        if buttons.len() >= LANGUAGES_LIMIT as usize {
            let removed = buttons.remove(0);
            self.languages_box.remove(&removed.1);
        }
        
        for button in buttons.iter() {
            button.1.remove_css_class("selected");
        }

        buttons.push(button.clone());
        
        if self.kind == LanguageButtonsKind::Source {
            if let Some(first_button) = self.languages_box.first_child()
                .and_then(|child| child.downcast::<gtk4::Button>().ok())
            {
                first_button.remove_css_class("selected");
                self.languages_box.insert_child_after(&button.1, Some(&first_button));
            }
            
            tokio::spawn(async move {
                let _ = translate::push_source_language(&language).await;
                let _ = state::set_source_language(&language.code).await;
            });
        } else {
            self.languages_box.prepend(&button.1);
            tokio::spawn(async move {
                let _ = translate::push_target_language(&language).await;
                let _ = state::set_target_language(&language.code).await;
            });
        }
        
        button.1.add_css_class("selected");
    }
    
    pub fn set_auto_language(&self, name: &str) {
        if let Some(button) = self.languages_box.first_child()
            .and_then(|child| child.downcast::<gtk4::Button>().ok())
        {
            button.set_label(&format!("Auto ({})", name));
        }
    }
}