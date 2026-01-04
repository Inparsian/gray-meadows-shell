use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;
use gtk4::prelude::*;
use relm4::RelmIterChildrenExt as _;

use crate::USERNAME;
use crate::config::read_config;
use crate::filesystem;
use crate::gesture;
use crate::singletons::ai;
use crate::widgets::common::loading;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ChatRole {
    User,
    Assistant
}

#[derive(Debug, Clone)]
pub struct ChatThinkingBlock {
    pub root: gtk4::Box,
    pub summary_root: gtk4cmark::view::MarkdownView,
    pub summary: Option<String>,
}

impl ChatThinkingBlock {
    pub fn new() -> Self {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.set_css_classes(&["ai-chat-thinking-block"]);

        let thinking_dropdown_button = gtk4::Button::new();
        thinking_dropdown_button.set_css_classes(&["ai-chat-thinking-dropdown-button"]);
        thinking_dropdown_button.set_valign(gtk4::Align::Start);
        thinking_dropdown_button.set_hexpand(true);
        root.append(&thinking_dropdown_button);

        let thinking_dropdown_header = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        thinking_dropdown_header.set_css_classes(&["ai-chat-thinking-dropdown-header"]);
        thinking_dropdown_header.set_hexpand(true);
        thinking_dropdown_button.set_child(Some(&thinking_dropdown_header));

        let thinking_dropdown_indicator = gtk4::Label::new(Some("lightbulb_2"));
        thinking_dropdown_indicator.set_css_classes(&["ai-chat-thinking-dropdown-indicator"]);
        thinking_dropdown_indicator.set_halign(gtk4::Align::Start);
        thinking_dropdown_indicator.set_xalign(0.0);
        thinking_dropdown_header.append(&thinking_dropdown_indicator);

        let thinking_dropdown_label = gtk4::Label::new(Some("Thoughts"));
        thinking_dropdown_label.set_css_classes(&["ai-chat-thinking-dropdown-label"]);
        thinking_dropdown_label.set_halign(gtk4::Align::Start);
        thinking_dropdown_label.set_xalign(0.0);
        thinking_dropdown_header.append(&thinking_dropdown_label);

        let thinking_dropdown_arrow = gtk4::Label::new(Some("stat_minus_1"));
        thinking_dropdown_arrow.set_css_classes(&["ai-chat-thinking-dropdown-arrow"]);
        thinking_dropdown_arrow.set_halign(gtk4::Align::End);
        thinking_dropdown_arrow.set_hexpand(true);
        thinking_dropdown_arrow.set_xalign(1.0);
        thinking_dropdown_header.append(&thinking_dropdown_arrow);

        let thinking_dropdown_revealer = gtk4::Revealer::new();
        thinking_dropdown_revealer.set_css_classes(&["ai-chat-thinking-dropdown-revealer"]);
        thinking_dropdown_revealer.set_transition_type(gtk4::RevealerTransitionType::SlideDown);
        thinking_dropdown_revealer.set_transition_duration(150);
        thinking_dropdown_revealer.set_reveal_child(false);
        root.append(&thinking_dropdown_revealer);

        let summary = gtk4cmark::view::MarkdownView::default();
        summary.set_css_classes(&["ai-chat-thinking-summary"]);
        summary.set_overflow(gtk4::Overflow::Hidden);
        summary.set_vexpand(true);
        summary.set_hexpand(true);
        thinking_dropdown_revealer.set_child(Some(&summary));

        thinking_dropdown_button.connect_clicked(move |_| {
            let currently_revealed = thinking_dropdown_revealer.reveals_child();
            thinking_dropdown_revealer.set_reveal_child(!currently_revealed);

            if currently_revealed {
                thinking_dropdown_arrow.remove_css_class("expanded");
            } else {
                thinking_dropdown_arrow.add_css_class("expanded");
            }
        });

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
    pub content: Option<String>,
    pub thinking: Option<ChatThinkingBlock>,
    pub root: gtk4::Box,
    pub markdown: gtk4cmark::view::MarkdownView,
    pub loading: gtk4::DrawingArea,
    pub header: gtk4::Box,
    pub footer: gtk4::Box,
}

impl ChatMessage {
    fn default_assistant_icon() -> gtk4::Widget {
        let sender_mui_icon = gtk4::Label::new(Some("robot"));
        sender_mui_icon.set_css_classes(&["ai-chat-message-sender-mui-icon"]);
        sender_mui_icon.set_halign(gtk4::Align::Start);
        sender_mui_icon.set_xalign(0.0);
        sender_mui_icon.upcast()
    }

    pub fn new(role: &ChatRole, content: Option<String>) -> Self {
        let app_config = read_config();
        let id = Rc::new(RefCell::new(None));
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.set_css_classes(&["ai-chat-message"]);
        root.set_valign(gtk4::Align::Start);

        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        header.set_css_classes(&["ai-chat-message-header"]);

        let sender_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        sender_box.set_css_classes(&["ai-chat-message-sender"]);

        let sender_icon: gtk4::Widget = match role {
            ChatRole::User => {
                let face_path = format!("{}/.face", filesystem::get_home_directory());
                if Path::new(&face_path).exists() {
                    let sender_face = gtk4::Image::new();
                    sender_face.set_css_classes(&["ai-chat-message-sender-icon"]);
                    sender_face.set_pixel_size(24);
                    sender_face.set_halign(gtk4::Align::Start);
                    sender_face.set_from_file(Some(face_path));
                    sender_face.upcast()
                } else {
                    let sender_mui_icon = gtk4::Label::new(Some("person"));
                    sender_mui_icon.set_css_classes(&["ai-chat-message-sender-mui-icon"]);
                    sender_mui_icon.set_halign(gtk4::Align::Start);
                    sender_mui_icon.set_xalign(0.0);
                    sender_mui_icon.upcast()
                }
            },
            
            ChatRole::Assistant => app_config.ai.assistant_icon_path.as_ref().map_or_else(|| {
                Self::default_assistant_icon()
            }, |icon_path| if Path::new(icon_path).exists() {
                let assistant_icon = gtk4::Image::new();
                assistant_icon.set_css_classes(&["ai-chat-message-sender-icon"]);
                assistant_icon.set_pixel_size(24);
                assistant_icon.set_halign(gtk4::Align::Start);
                assistant_icon.set_from_file(Some(icon_path));
                assistant_icon.upcast()
            } else {
                Self::default_assistant_icon()
            }),
        };

        let sender_label = gtk4::Label::new(Some(match role {
            ChatRole::User => &USERNAME,
            ChatRole::Assistant => app_config.ai.assistant_name.as_ref().map_or("AI Assistant", |name| name.as_str()),
        }));
        sender_label.set_css_classes(&["ai-chat-message-sender-label"]);
        sender_label.set_halign(gtk4::Align::Start);
        sender_label.set_xalign(0.0);

        sender_box.append(&sender_icon);
        sender_box.append(&sender_label);
        header.append(&sender_box);

        let controls_revealer = gtk4::Revealer::new();
        controls_revealer.set_css_classes(&["ai-chat-message-controls-revealer"]);
        controls_revealer.set_halign(gtk4::Align::End);
        controls_revealer.set_valign(gtk4::Align::Start);
        controls_revealer.set_hexpand(true);
        controls_revealer.set_transition_type(gtk4::RevealerTransitionType::Crossfade);
        controls_revealer.set_transition_duration(150);

        let controls_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        controls_box.set_css_classes(&["ai-chat-message-controls-box"]);
        controls_revealer.set_child(Some(&controls_box));

        let delete_button = gtk4::Button::new();
        delete_button.set_css_classes(&["ai-chat-message-control-button"]);
        delete_button.set_label("delete");
        delete_button.connect_clicked({
            let id = id.clone();
            move |_| if !ai::is_currently_in_cycle() && let Some(message_id) = *id.borrow() {
                ai::trim_items(message_id);
            }
        });
        controls_box.append(&delete_button);

        let retry_button = gtk4::Button::new();
        retry_button.set_css_classes(&["ai-chat-message-control-button"]);
        retry_button.set_label("refresh");
        retry_button.connect_clicked({
            let id = id.clone();
            let role = role.clone();
            move |_| if !ai::is_currently_in_cycle() && let Some(message_id) = *id.borrow() {
                // Increase message_id by 1 if this is a user message to trim down to the
                // assistant response directly after it
                let message_id = if role == ChatRole::User {
                    message_id + 1
                } else {
                    message_id
                };

                ai::trim_items(message_id);
                tokio::spawn(ai::start_request_cycle());
            }
        });
        controls_box.append(&retry_button);

        header.append(&controls_revealer);

        let markdown = gtk4cmark::view::MarkdownView::default();
        markdown.set_css_classes(&["ai-chat-message-content"]);
        markdown.set_overflow(gtk4::Overflow::Hidden);
        markdown.set_vexpand(true);
        markdown.set_hexpand(true);

        let loading = loading::new();
        loading.set_halign(gtk4::Align::Start);
        loading.set_valign(gtk4::Align::Start);

        // This will start out with empty content, to be filled in later
        let footer = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        footer.set_css_classes(&["ai-chat-message-footer"]);
        footer.set_valign(gtk4::Align::End);

        root.append(&header);
        root.append(&footer);
        
        if let Some(initial_content) = &content {
            root.insert_child_after(&markdown, Some(&header));
            markdown.set_markdown(initial_content.as_str());
        } else {
            root.insert_child_after(&loading, Some(&header));
        }

        root.add_controller(gesture::on_enter({
            let controls_revealer = controls_revealer.clone();
            move |_, _| {
                controls_revealer.set_reveal_child(true);
            }
        }));

        root.add_controller(gesture::on_leave(move || {
            controls_revealer.set_reveal_child(false);
        }));

        Self {
            id,
            content,
            thinking: None,
            root,
            markdown,
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
        self.markdown.set_markdown(content);

        if was_none {
            // If thinking is present, insert after that instead
            if let Some(thinking_block) = &self.thinking {
                self.root.insert_child_after(&self.markdown, Some(&thinking_block.root));
            } else {
                self.root.insert_child_after(&self.markdown, Some(&self.header));
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

#[derive(Debug, Clone)]
pub struct Chat {
    pub messages: Rc<RefCell<Vec<ChatMessage>>>,
    pub root: gtk4::Box,
}

impl Default for Chat {
    fn default() -> Self {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        root.set_css_classes(&["ai-chat-messages"]);
        root.set_valign(gtk4::Align::Start);

        Self {
            messages: Rc::new(RefCell::new(Vec::new())),
            root,
        }
    }
}

impl Chat {
    pub fn clear_messages(&self) {
        self.messages.borrow_mut().clear();
        self.root.iter_children().for_each(|child| {
            self.root.remove(&child);
        });
    }

    pub fn trim_messages(&self, down_to_id: i64) {
        let mut messages = self.messages.borrow_mut();
        let mut ids_to_remove = Vec::new();

        for message in messages.iter() {
            if let Some(id) = *message.id.borrow() && id >= down_to_id {
                ids_to_remove.push(id);
            }
        }

        messages.retain(|message| {
            if let Some(id) = *message.id.borrow() && ids_to_remove.contains(&id) {
                self.root.remove(&message.root);
                return false;
            }
            true
        });
    }

    pub fn add_message(&self, message: ChatMessage) {
        self.root.append(&message.root);
        self.messages.borrow_mut().push(message);
    }

    pub fn remove_latest_message(&self) -> Option<ChatMessage> {
        if let Some(message) = self.messages.borrow_mut().pop() {
            self.root.remove(&message.root);
            Some(message)
        } else {
            None
        }
    }

    pub fn append_tool_call_to_latest_message(&self, tool_name: &str, arguments: &str) {
        let mut messages = self.messages.borrow_mut();
        if let Some(latest_message) = messages.last_mut() {
            let tool_call_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
            tool_call_box.set_css_classes(&["ai-chat-message-tool-call"]);

            let tool_name_label = gtk4::Label::new(Some(tool_name));
            tool_name_label.set_css_classes(&["ai-chat-message-tool-call-name"]);
            tool_name_label.set_halign(gtk4::Align::Start);
            tool_name_label.set_xalign(0.0);

            let arguments_label = gtk4::Label::new(Some(arguments));
            arguments_label.set_css_classes(&["ai-chat-message-tool-call-arguments"]);
            arguments_label.set_halign(gtk4::Align::Start);
            arguments_label.set_xalign(0.0);

            tool_call_box.append(&tool_name_label);
            tool_call_box.append(&arguments_label);

            latest_message.footer.append(&tool_call_box);
            
            if latest_message.content.is_none() {
                latest_message.set_content("");
            }
        }
    }

    pub fn append_thinking_block_to_latest_message(&self, summary: &str) {
        let mut messages = self.messages.borrow_mut();
        if let Some(latest_message) = messages.last_mut() {
            latest_message.set_thinking(summary);
        }
    }
}