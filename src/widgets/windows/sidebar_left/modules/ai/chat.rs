use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;
use gtk4::prelude::*;

use crate::filesystem;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ChatRole {
    User,
    Assistant
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub content: String,
    pub root: gtk4::Box,
    pub markdown: gtk4cmark::view::MarkdownView,
    pub footer: gtk4::Box,
}

impl ChatMessage {
    pub fn new(role: &ChatRole, content: String) -> Self {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.set_css_classes(&["ai-chat-message"]);
        root.set_valign(gtk4::Align::Start);

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
            
            ChatRole::Assistant => {
                let sender_mui_icon = gtk4::Label::new(Some("robot"));
                sender_mui_icon.set_css_classes(&["ai-chat-message-sender-mui-icon"]);
                sender_mui_icon.set_halign(gtk4::Align::Start);
                sender_mui_icon.set_xalign(0.0);
                sender_mui_icon.upcast()
            },
        };

        let sender_label = gtk4::Label::new(Some(match role {
            ChatRole::User => "You",
            ChatRole::Assistant => "AI Assistant"
        }));
        sender_label.set_css_classes(&["ai-chat-message-sender-label"]);
        sender_label.set_halign(gtk4::Align::Start);
        sender_label.set_xalign(0.0);

        sender_box.append(&sender_icon);
        sender_box.append(&sender_label);

        let markdown = gtk4cmark::view::MarkdownView::default();
        markdown.set_css_classes(&["ai-chat-message-content"]);
        markdown.set_markdown(content.as_str());
        markdown.set_overflow(gtk4::Overflow::Hidden);
        markdown.set_vexpand(true);
        markdown.set_hexpand(true);

        // This will start out with empty content, to be filled in later
        let footer = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        footer.set_css_classes(&["ai-chat-message-footer"]);
        footer.set_valign(gtk4::Align::End);

        root.append(&sender_box);
        root.append(&markdown);
        root.append(&footer);

        Self {
            content,
            root,
            markdown,
            footer,
        }
    }

    pub fn set_content(&mut self, content: &str) {
        self.content = content.to_owned();
        self.markdown.set_markdown(content);
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
    pub fn add_message(&self, message: ChatMessage) {
        self.root.append(&message.root);
        self.messages.borrow_mut().push(message);
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
        }
    }
}