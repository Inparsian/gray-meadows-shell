use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;
use gtk4::prelude::*;
use relm4::RelmIterChildrenExt as _;

use crate::APP;
use crate::USERNAME;
use crate::filesystem;
use crate::gesture;
use crate::singletons::openai;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ChatRole {
    User,
    Assistant
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub id: Rc<RefCell<Option<i64>>>,
    pub content: String,
    pub root: gtk4::Box,
    pub markdown: gtk4cmark::view::MarkdownView,
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

    pub fn new(role: &ChatRole, content: String) -> Self {
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
            
            ChatRole::Assistant => APP.config.ai.assistant_icon_path.as_ref().map_or_else(|| {
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
            ChatRole::Assistant => APP.config.ai.assistant_name.as_ref().map_or("AI Assistant", |name| name.as_str()),
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

        let delete_button = gtk4::Button::new();
        delete_button.set_css_classes(&["ai-chat-message-delete-button"]);
        delete_button.set_label("delete");
        delete_button.connect_clicked({
            let id = id.clone();
            move |_| if !openai::is_currently_in_cycle()
                && let Some(message_id) = *id.borrow() 
            {
                openai::trim_messages(message_id);
            }
        });
        controls_revealer.set_child(Some(&delete_button));

        header.append(&controls_revealer);

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

        root.append(&header);
        root.append(&markdown);
        root.append(&footer);

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
            root,
            markdown,
            footer,
        }
    }

    pub fn set_content(&mut self, content: &str) {
        self.content = content.to_owned();
        self.markdown.set_markdown(content);
    }

    pub fn set_id(&self, id: i64) {
        *self.id.borrow_mut() = Some(id);
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