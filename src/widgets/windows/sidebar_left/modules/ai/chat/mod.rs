pub mod message;
pub mod content;

use std::rc::Rc;
use std::cell::RefCell;
use gtk::prelude::*;
use relm4::RelmIterChildrenExt as _;

use crate::services::ai::images::uuid_to_file_path;
use self::message::{ChatMessage, ChatRole};

#[derive(Debug, Clone)]
pub struct Chat {
    pub messages: Rc<RefCell<Vec<ChatMessage>>>,
    pub bx: gtk::Box,
    pub root: gtk::Viewport,
}

impl Default for Chat {
    fn default() -> Self {
        let bx = gtk::Box::new(gtk::Orientation::Vertical, 8);
        bx.set_css_classes(&["ai-chat-messages"]);
        bx.set_valign(gtk::Align::Start);

        let root = gtk::Viewport::default();
        root.set_vscroll_policy(gtk::ScrollablePolicy::Natural);
        root.set_child(Some(&bx));

        Self {
            messages: Rc::new(RefCell::new(Vec::new())),
            bx,
            root,
        }
    }
}

impl Chat {
    pub fn clear_messages(&self) {
        self.messages.borrow_mut().clear();
        self.bx.iter_children().for_each(|child| {
            self.bx.remove(&child);
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
                self.bx.remove(&message.root);
                return false;
            }
            true
        });
    }

    pub fn add_message(&self, message: ChatMessage) {
        self.bx.append(&message.root);
        self.messages.borrow_mut().push(message);
    }

    pub fn remove_latest_message(&self) -> Option<ChatMessage> {
        if let Some(message) = self.messages.borrow_mut().pop() {
            self.bx.remove(&message.root);
            Some(message)
        } else {
            None
        }
    }

    pub fn assert_last_message_is_role(&self, expected_role: ChatRole, id: Option<i64>) {
        let messages = self.messages.borrow();
        if let Some(latest_message) = messages.last() {
            if latest_message.role != expected_role {
                // Add a new message of the expected role
                let new_message = ChatMessage::new(expected_role, None);
                drop(messages);
                self.add_message(new_message);
                if let Some(id) = id {
                    self.messages.borrow_mut().last().unwrap().set_id(id);
                }
            }
        } else {
            let new_message = ChatMessage::new(expected_role, None);
            drop(messages);
            self.add_message(new_message);
            if let Some(id) = id {
                self.messages.borrow_mut().last().unwrap().set_id(id);
            }
        }
    }

    pub fn append_tool_call_to_latest_message(&self, tool_name: &str, arguments: &str) {
        let mut messages = self.messages.borrow_mut();
        if let Some(latest_message) = messages.last_mut() {
            let tool_call_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
            tool_call_box.set_css_classes(&["ai-chat-message-tool-call"]);

            let tool_name_label = gtk::Label::new(Some(tool_name));
            tool_name_label.set_css_classes(&["ai-chat-message-tool-call-name"]);
            tool_name_label.set_halign(gtk::Align::Start);
            tool_name_label.set_xalign(0.0);

            let arguments_label = gtk::Label::new(Some(arguments));
            arguments_label.set_css_classes(&["ai-chat-message-tool-call-arguments"]);
            arguments_label.set_halign(gtk::Align::Start);
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

    pub fn append_image_to_latest_message(&self, uuid: &str) {
        let mut messages = self.messages.borrow_mut();
        if let Some(latest_message) = messages.last_mut() {
            match gtk::gdk::Texture::from_filename(uuid_to_file_path(uuid)) {
                Ok(texture) => {
                    let w_clamp = libadwaita::Clamp::new();
                    w_clamp.set_maximum_size(300);
                    w_clamp.set_unit(libadwaita::LengthUnit::Px);
                    w_clamp.set_halign(gtk::Align::Start);
                    w_clamp.set_valign(gtk::Align::Start);
                    
                    let h_clamp = libadwaita::Clamp::new();
                    h_clamp.set_maximum_size(300);
                    h_clamp.set_unit(libadwaita::LengthUnit::Px);
                    h_clamp.set_orientation(gtk::Orientation::Vertical);
                    w_clamp.set_child(Some(&h_clamp));

                    let picture = gtk::Picture::new();
                    picture.set_css_classes(&["ai-chat-message-image"]);
                    picture.set_paintable(Some(&texture));
                    picture.set_content_fit(gtk::ContentFit::ScaleDown);
                    h_clamp.set_child(Some(&picture));
                    latest_message.footer.append(&w_clamp);
                    *latest_message.attachments.borrow_mut() += 1;

                    if latest_message.content.is_none() {
                        latest_message.set_content("");
                    }
                },

                Err(err) => {
                    error!(uuid, ?err, "Failed to load image");
                }
            }
        }
    }
}