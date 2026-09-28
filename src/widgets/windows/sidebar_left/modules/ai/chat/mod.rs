pub mod message;
pub mod content;
pub mod elements;

use std::rc::Rc;
use std::cell::RefCell;
use gtk::prelude::*;
use relm4::RelmIterChildrenExt as _;

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
            if let Some(id) = message.id.get() && id >= down_to_id {
                ids_to_remove.push(id);
            }
        }

        messages.retain(|message| {
            if let Some(id) = message.id.get() && ids_to_remove.contains(&id) {
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

    pub fn latest_message(&self) -> Option<ChatMessage> {
        self.messages.borrow().last().cloned()
    }

    pub fn message_for(&self, role: ChatRole, item_id: Option<i64>) -> ChatMessage {
        let message = self.latest_message()
            .filter(|latest| latest.role == role && !latest.closed.get())
            .unwrap_or_else(|| {
                let message = ChatMessage::new(role);
                self.add_message(message.clone());
                message
            });

        if let Some(item_id) = item_id {
            message.record_item(item_id);
        }

        message
    }

    pub fn close_latest(&self) {
        if let Some(latest) = self.messages.borrow().last() {
            latest.closed.set(true);
        }
    }
}
