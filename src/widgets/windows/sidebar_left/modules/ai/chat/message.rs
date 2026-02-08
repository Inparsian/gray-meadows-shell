use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;
use gtk::prelude::*;

use crate::USERNAME;
use crate::config::read_config;
use crate::services::ai;
use crate::services::ai::types::AiConversationItemPayload;
use crate::utils::{filesystem, gesture};
use crate::widgets::common::loading;
use crate::widgets::common::revealer::{AdwRevealer, AdwRevealerDirection, GEasing};
use super::content::ChatMessageContent;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ChatRole {
    User,
    Assistant
}

#[derive(Debug, Clone)]
pub struct ChatThinkingBlock {
    pub root: gtk::Box,
    pub summary_root: gtk4cmark::MarkdownView,
    pub summary: Option<String>,
}

impl ChatThinkingBlock {
    pub fn new() -> Self {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root.set_css_classes(&["ai-chat-thinking-block"]);

        let thinking_dropdown_button = gtk::Button::new();
        thinking_dropdown_button.set_css_classes(&["ai-chat-thinking-dropdown-button"]);
        thinking_dropdown_button.set_valign(gtk::Align::Start);
        thinking_dropdown_button.set_hexpand(true);
        root.append(&thinking_dropdown_button);

        let thinking_dropdown_header = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        thinking_dropdown_header.set_css_classes(&["ai-chat-thinking-dropdown-header"]);
        thinking_dropdown_header.set_hexpand(true);
        thinking_dropdown_button.set_child(Some(&thinking_dropdown_header));

        let thinking_dropdown_indicator = gtk::Label::new(Some("lightbulb_2"));
        thinking_dropdown_indicator.set_css_classes(&["ai-chat-thinking-dropdown-indicator"]);
        thinking_dropdown_indicator.set_halign(gtk::Align::Start);
        thinking_dropdown_indicator.set_xalign(0.0);
        thinking_dropdown_header.append(&thinking_dropdown_indicator);

        let thinking_dropdown_label = gtk::Label::new(Some("Thoughts"));
        thinking_dropdown_label.set_css_classes(&["ai-chat-thinking-dropdown-label"]);
        thinking_dropdown_label.set_halign(gtk::Align::Start);
        thinking_dropdown_label.set_xalign(0.0);
        thinking_dropdown_header.append(&thinking_dropdown_label);

        let thinking_dropdown_arrow = gtk::Label::new(Some("stat_minus_1"));
        thinking_dropdown_arrow.set_css_classes(&["ai-chat-thinking-dropdown-arrow"]);
        thinking_dropdown_arrow.set_halign(gtk::Align::End);
        thinking_dropdown_arrow.set_hexpand(true);
        thinking_dropdown_arrow.set_xalign(1.0);
        thinking_dropdown_header.append(&thinking_dropdown_arrow);

        let thinking_dropdown_revealer = AdwRevealer::default();
        thinking_dropdown_revealer.set_css_classes(&["ai-chat-thinking-dropdown-revealer"]);
        thinking_dropdown_revealer.set_transition_direction(AdwRevealerDirection::Down);
        thinking_dropdown_revealer.set_show_easing(GEasing::EaseOutExpo);
        thinking_dropdown_revealer.set_hide_easing(GEasing::EaseOutExpo);
        thinking_dropdown_revealer.set_transition_duration(500);
        thinking_dropdown_revealer.set_reveal(false);
        root.append(&thinking_dropdown_revealer);

        let summary = gtk4cmark::MarkdownView::default();
        summary.set_css_classes(&["ai-chat-thinking-summary"]);
        summary.set_overflow(gtk::Overflow::Hidden);
        summary.set_vexpand(true);
        summary.set_hexpand(true);
        thinking_dropdown_revealer.set_child_from(Some(&summary));

        thinking_dropdown_button.connect_clicked(clone!(
            #[weak] root,
            move |_| {
                let currently_revealed = thinking_dropdown_revealer.reveal();
                thinking_dropdown_revealer.set_reveal(!currently_revealed);
                
                if currently_revealed {
                    root.remove_css_class("expanded");
                } else {
                    root.add_css_class("expanded");
                }
            }
        ));

        Self {
            root,
            summary_root: summary,
            summary: None,
        }
    }

    pub fn set_summary(&mut self, content: &str) {
        self.summary_root.set_markdown(content);
        self.summary = Some(content.to_owned());
    }
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub id: Rc<RefCell<Option<i64>>>,
    pub role: ChatRole,
    pub content: Option<String>,
    pub thinking: Option<ChatThinkingBlock>,
    pub attachments: Rc<RefCell<i64>>,
    pub root: gtk::Box,
    pub view: ChatMessageContent,
    pub loading: gtk::DrawingArea,
    pub header: gtk::Box,
    pub footer: gtk::Box,
}

impl ChatMessage {
    fn default_assistant_icon() -> gtk::Widget {
        let sender_mui_icon = gtk::Label::new(Some("robot"));
        sender_mui_icon.set_css_classes(&["ai-chat-message-sender-mui-icon"]);
        sender_mui_icon.set_halign(gtk::Align::Start);
        sender_mui_icon.set_xalign(0.0);
        sender_mui_icon.upcast()
    }

    pub fn new(role: ChatRole, content: Option<String>) -> Self {
        let app_config = read_config();
        let id = Rc::new(RefCell::new(None));
        let attachments = Rc::new(RefCell::new(0));
        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root.set_css_classes(&["ai-chat-message"]);
        root.set_valign(gtk::Align::Start);

        let header = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        header.set_css_classes(&["ai-chat-message-header"]);

        let sender_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        sender_box.set_css_classes(&["ai-chat-message-sender"]);

        let sender_icon: gtk::Widget = match role {
            ChatRole::User => {
                let face_path = format!("{}/.face", filesystem::get_home_directory());
                if Path::new(&face_path).exists() {
                    let sender_face = gtk::Image::new();
                    sender_face.set_css_classes(&["ai-chat-message-sender-icon"]);
                    sender_face.set_pixel_size(24);
                    sender_face.set_halign(gtk::Align::Start);
                    sender_face.set_from_file(Some(face_path));
                    sender_face.upcast()
                } else {
                    let sender_mui_icon = gtk::Label::new(Some("person"));
                    sender_mui_icon.set_css_classes(&["ai-chat-message-sender-mui-icon"]);
                    sender_mui_icon.set_halign(gtk::Align::Start);
                    sender_mui_icon.set_xalign(0.0);
                    sender_mui_icon.upcast()
                }
            },
            
            ChatRole::Assistant => app_config.ai.assistant_icon_path.as_ref().map_or_else(|| {
                Self::default_assistant_icon()
            }, |icon_path| if Path::new(icon_path).exists() {
                let assistant_icon = gtk::Image::new();
                assistant_icon.set_css_classes(&["ai-chat-message-sender-icon"]);
                assistant_icon.set_pixel_size(24);
                assistant_icon.set_halign(gtk::Align::Start);
                assistant_icon.set_from_file(Some(icon_path));
                assistant_icon.upcast()
            } else {
                Self::default_assistant_icon()
            }),
        };

        let sender_label = gtk::Label::new(Some(match role {
            ChatRole::User => &USERNAME,
            ChatRole::Assistant => app_config.ai.assistant_name.as_ref().map_or("AI Assistant", |name| name.as_str()),
        }));
        sender_label.set_css_classes(&["ai-chat-message-sender-label"]);
        sender_label.set_halign(gtk::Align::Start);
        sender_label.set_xalign(0.0);

        sender_box.append(&sender_icon);
        sender_box.append(&sender_label);
        header.append(&sender_box);

        let controls_revealer = gtk::Revealer::new();
        controls_revealer.set_css_classes(&["ai-chat-message-controls-revealer"]);
        controls_revealer.set_halign(gtk::Align::End);
        controls_revealer.set_valign(gtk::Align::Start);
        controls_revealer.set_hexpand(true);
        controls_revealer.set_transition_type(gtk::RevealerTransitionType::Crossfade);
        controls_revealer.set_transition_duration(150);

        let controls_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        controls_box.set_css_classes(&["ai-chat-message-controls-box"]);
        controls_revealer.set_child(Some(&controls_box));
        
        let view = ChatMessageContent::new();
        view.connect_closure("save-edit", false, closure_local!(
            #[strong] id,
            move |view: ChatMessageContent| {
                let content = view.content();
                if !ai::is_currently_in_cycle() && let Some(message_id) = *id.borrow() {
                    let first_encountered_message = ai::get_first_encountered_message_payload(message_id);
                    
                    if let Some((item_id, AiConversationItemPayload::Message { id, role, thought_signature, .. })) = first_encountered_message {
                        let payload = AiConversationItemPayload::Message {
                            id,
                            content,
                            role,
                            thought_signature,
                        };
                        
                        tokio::spawn(ai::update_item(item_id, payload));
                    }
                }
            }
        ));

        let delete_button = gtk::Button::new();
        delete_button.set_css_classes(&["ai-chat-message-control-button"]);
        delete_button.set_label("delete");
        delete_button.connect_clicked(clone!(
            #[strong] id,
            move |_| if !ai::is_currently_in_cycle() && let Some(message_id) = *id.borrow() {
                glib::spawn_future_local(ai::trim_items(message_id));
            }
        ));
        controls_box.append(&delete_button);
        
        let edit_button = gtk::Button::new();
        edit_button.set_css_classes(&["ai-chat-message-control-button"]);
        edit_button.set_label("edit");
        edit_button.connect_clicked(clone!(
            #[weak] view,
            move |_| if !ai::is_currently_in_cycle() {
                view.set_editing(true);
            }
        ));
        controls_box.append(&edit_button);

        let retry_button = gtk::Button::new();
        retry_button.set_css_classes(&["ai-chat-message-control-button"]);
        retry_button.set_label("refresh");
        retry_button.connect_clicked(clone!(
            #[strong] id,
            #[strong] attachments,
            #[strong] role,
            move |_| if !ai::is_currently_in_cycle() && let Some(message_id) = *id.borrow() {
                // Increase message_id by 1 if this is a user message to trim down to the
                // assistant response directly after it
                let message_id = if role == ChatRole::User {
                    message_id + 1 + *attachments.borrow()
                } else {
                    message_id
                };

                tokio::spawn(async move {
                    ai::trim_items(message_id).await;
                    ai::start_request_cycle().await;
                });
            }
        ));
        controls_box.append(&retry_button);

        header.append(&controls_revealer);
        
        let loading = loading::new();
        loading.set_halign(gtk::Align::Start);
        loading.set_valign(gtk::Align::Start);

        // This will start out with empty content, to be filled in later
        let footer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        footer.set_css_classes(&["ai-chat-message-footer"]);
        footer.set_valign(gtk::Align::End);

        root.append(&header);
        root.append(&footer);
        
        if let Some(initial_content) = &content {
            root.insert_child_after(&view, Some(&header));
            view.set_content(initial_content.as_str());
        } else {
            root.insert_child_after(&loading, Some(&header));
        }

        root.add_controller(gesture::on_enter(clone!(
            #[weak] controls_revealer,
            move |_, _| {
                controls_revealer.set_reveal_child(true);
            }
        )));

        root.add_controller(gesture::on_leave(move || {
            controls_revealer.set_reveal_child(false);
        }));

        Self {
            id,
            role,
            content,
            thinking: None,
            attachments,
            root,
            view,
            loading,
            header,
            footer,
        }
    }

    pub fn set_id(&self, id: i64) {
        *self.id.borrow_mut() = Some(id);
    }

    pub fn set_content(&mut self, content: &str) {
        let was_none = self.content.is_none();

        self.content = Some(content.to_owned());
        self.view.set_content(content);

        if was_none {
            // If thinking is present, insert after that instead
            if let Some(thinking_block) = &self.thinking {
                self.root.insert_child_after(&self.view, Some(&thinking_block.root));
            } else {
                self.root.insert_child_after(&self.view, Some(&self.header));
            }

            self.root.remove(&self.loading);
        }
    }

    pub fn set_thinking(&mut self, content: &str) {
        let was_none = self.thinking.is_none();
        if was_none {
            let mut thinking_block = ChatThinkingBlock::new();
            thinking_block.set_summary(content);
            self.thinking = Some(thinking_block.clone());
            self.root.insert_child_after(&thinking_block.root, Some(&self.header));
        } else if let Some(thinking_block) = &mut self.thinking {
            thinking_block.set_summary(content);
        }
    }
}