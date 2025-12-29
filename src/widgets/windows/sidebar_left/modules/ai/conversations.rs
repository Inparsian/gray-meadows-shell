use std::cell::RefCell;
use std::rc::Rc;
use gtk4::prelude::*;

use crate::sql::wrappers::aichats::{self, SqlAiConversation};
use crate::singletons::openai;
use crate::gesture;

fn conversation_control_button(icon_name: &str, tooltip: &str) -> gtk4::Button {
    let button = gtk4::Button::new();
    button.set_css_classes(&["ai-chat-conversation-item-control-button"]);
    button.set_label(icon_name);
    button.set_tooltip_text(Some(tooltip));

    button
}

#[derive(Debug, Clone)]
pub struct ConversationItem {
    pub conversation: Rc<RefCell<SqlAiConversation>>,
    pub root: gtk4::Box,
    pub title_label: gtk4::Label,
    pub length_label: gtk4::Label,
}

impl ConversationItem {
    pub fn new(conversation: SqlAiConversation) -> Self {
        let conversation = Rc::new(RefCell::new(conversation));

        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        root.set_css_classes(&["ai-chat-conversation-item"]);

        let info_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        info_box.set_css_classes(&["ai-chat-conversation-item-info-box"]);
        root.append(&info_box);
        
        let title_label = gtk4::Label::new(Some(&conversation.borrow().title));
        title_label.set_css_classes(&["ai-chat-conversation-item-title-label"]);
        title_label.set_halign(gtk4::Align::Start);
        title_label.set_xalign(0.0);
        info_box.append(&title_label);

        let title_input = gtk4::Entry::new();
        title_input.set_text(&conversation.borrow().title);
        title_input.set_css_classes(&["ai-chat-conversation-item-title-input"]);
        title_input.set_halign(gtk4::Align::Start);
        title_input.set_visible(false);
        title_input.connect_activate({
            let conversation = conversation.clone();
            let title_label = title_label.clone();
            let title_input = title_input.clone();
            move |_| {
                let new_title = title_input.text().to_string();
                if !new_title.is_empty() && new_title != conversation.borrow().title {
                    openai::conversation::rename_conversation(conversation.borrow().id, &new_title);
                    conversation.borrow_mut().title = new_title;
                    title_input.set_visible(false);
                    title_label.set_visible(true);
                }
            }
        });
        info_box.append(&title_input);

        let message_count = aichats::get_messages_length(conversation.borrow().id).unwrap_or(0);
        let length_label = gtk4::Label::new(Some(&format!("{} messages", message_count)));
        length_label.set_css_classes(&["ai-chat-conversation-item-length-label"]);
        length_label.set_halign(gtk4::Align::Start);
        length_label.set_xalign(0.0);
        info_box.append(&length_label);

        let controls_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        controls_box.set_css_classes(&["ai-chat-conversation-item-controls-box"]);

        let load_button = conversation_control_button("download", "Load Conversation");
        load_button.connect_clicked({
            let conversation = conversation.clone();
            move |_| {
                openai::conversation::load_conversation(conversation.borrow().id);
            }
        });
        controls_box.append(&load_button);

        let rename_button = conversation_control_button("edit", "Rename Conversation");
        rename_button.connect_clicked({
            let title_label = title_label.clone();
            move |_| {
                title_label.set_visible(false);
                title_input.set_visible(true);
                title_input.grab_focus();
            }
        });
        controls_box.append(&rename_button);

        let delete_button = conversation_control_button("close", "Delete Conversation");
        delete_button.connect_clicked({
            let conversation = conversation.clone();
            move |_| {
                openai::conversation::delete_conversation(conversation.borrow().id);
            }
        });
        controls_box.append(&delete_button);

        let controls_revealer = gtk4::Revealer::new();
        controls_revealer.set_css_classes(&["ai-chat-conversation-item-controls-revealer"]);
        controls_revealer.set_halign(gtk4::Align::End);
        controls_revealer.set_valign(gtk4::Align::Start);
        controls_revealer.set_hexpand(true);
        controls_revealer.set_transition_type(gtk4::RevealerTransitionType::SlideLeft);
        controls_revealer.set_transition_duration(200);
        controls_revealer.set_child(Some(&controls_box));
        root.append(&controls_revealer);

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
            conversation,
            root,
            title_label,
            length_label,
        }
    }

    pub fn set_title(&self, new_title: &str) {
        self.title_label.set_text(new_title);
    }

    pub fn set_length(&self, new_length: usize) {
        self.length_label.set_text(&format!("{} messages", new_length));
    }
}

#[derive(Debug, Clone)]
pub struct ConversationsList {
    pub root: gtk4::Box,
    pub conversations: Rc<RefCell<Vec<ConversationItem>>>,
}

impl ConversationsList {
    pub fn new() -> Self {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.set_css_classes(&["ai-chat-conversations-list"]);

        let me = Self {
            root,
            conversations: Rc::new(RefCell::new(Vec::new())),
        };

        // Listen for events from the AI singleton channel
        let receiver = openai::CHANNEL.get().map(|channel| channel.subscribe());
        if let Some(mut receiver) = receiver {
            gtk4::glib::spawn_future_local({
                let me = me.clone();
                async move {
                    while let Ok(message) = receiver.recv().await {
                        match message {
                            openai::AIChannelMessage::ConversationAdded(conversation) => {
                                let item = ConversationItem::new(conversation);
                                me.root.append(&item.root);
                                me.conversations.borrow_mut().push(item);
                            },

                            openai::AIChannelMessage::ConversationDeleted(conversation_id) => {
                                let mut conversations = me.conversations.borrow_mut();
                                if let Some(pos) = conversations.iter().position(|item| item.conversation.borrow().id == conversation_id) {
                                    let item = conversations.remove(pos);
                                    me.root.remove(&item.root);
                                }
                            },

                            openai::AIChannelMessage::ConversationRenamed(conversation_id, new_title) => {
                                let conversations = me.conversations.borrow();
                                for item in conversations.iter() {
                                    if item.conversation.borrow().id == conversation_id {
                                        item.set_title(&new_title);
                                        break;
                                    }
                                }
                            },

                            openai::AIChannelMessage::ConversationLoaded(conversation) => {
                                let conversations = me.conversations.borrow();
                                for item in conversations.iter() {
                                    if item.conversation.borrow().id == conversation.id {
                                        let current_length = aichats::get_messages_length(conversation.id).unwrap_or(0);
                                        item.set_length(current_length);
                                        break;
                                    }
                                }
                            },

                            openai::AIChannelMessage::CycleStarted |
                            openai::AIChannelMessage::CycleFinished => {
                                let Some(current_conversation) = openai::SESSION.get().and_then(|session| {
                                    let conversation = session.conversation.read().unwrap();
                                    conversation.clone()
                                }) else {
                                    continue;
                                };

                                let conversations = me.conversations.borrow();
                                for item in conversations.iter() {
                                    if item.conversation.borrow().id == current_conversation.id {
                                        let current_length = aichats::get_messages_length(current_conversation.id).unwrap_or(0);
                                        item.set_length(current_length);
                                        break;
                                    }
                                }
                            }

                            _ => {},
                        }
                    }
                }
            });
        }

        // Add existing conversations from the database
        if let Ok(existing_conversations) = aichats::get_all_conversations() {
            for conversation in existing_conversations {
                let item = ConversationItem::new(conversation);
                me.root.append(&item.root);
                me.conversations.borrow_mut().push(item);
            }
        }

        me
    }
}