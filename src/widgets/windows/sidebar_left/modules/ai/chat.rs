use std::rc::Rc;
use std::cell::RefCell;
use gtk4::prelude::*;

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
}

impl ChatMessage {
    pub fn new(role: &ChatRole, content: String) -> Self {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.set_css_classes(&["ai-chat-message"]);
        root.set_valign(gtk4::Align::Start);

        let sender_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        sender_box.set_css_classes(&["ai-chat-message-sender"]);

        let sender_icon = match role {
            ChatRole::User => gtk4::Label::new(Some("person")),
            ChatRole::Assistant => gtk4::Label::new(Some("robot")),
        };
        sender_icon.set_css_classes(&["ai-chat-message-sender-icon"]);
        sender_icon.set_halign(gtk4::Align::Start);
        sender_icon.set_xalign(0.0);

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

        root.append(&sender_box);
        root.append(&markdown);

        Self {
            content,
            root,
            markdown,
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
}