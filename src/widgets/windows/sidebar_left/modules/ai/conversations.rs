use std::cell::RefCell;
use std::rc::Rc;
use gtk::prelude::*;

use crate::sql::wrappers::aichats;
use crate::services::ai::{self, AiChannelMessage, types::AiConversation};
use crate::utils::gesture;

fn conversation_control_button(icon_name: &str, tooltip: &str) -> gtk::Button {
    let button = gtk::Button::new();
    button.set_css_classes(&["ai-chat-conversation-item-control-button"]);
    button.set_label(icon_name);
    button.set_tooltip_text(Some(tooltip));

    button
}

fn message_count_str(count: usize) -> String {
    match count {
        0 => "No messages".to_owned(),
        1 => "1 message".to_owned(),
        n => format!("{} messages", n),
    }
}

#[derive(Debug, Clone)]
pub struct ConversationItem {
    pub conversation: Rc<RefCell<AiConversation>>,
    pub root: gtk::Box,
    pub title_label: gtk::Label,
    pub length_label: gtk::Label,
}

impl ConversationItem {
    pub async fn new(conversation: AiConversation) -> Self {
        let conversation = Rc::new(RefCell::new(conversation));

        let root = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        root.set_css_classes(&["ai-chat-conversation-item"]);

        let info_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        info_box.set_css_classes(&["ai-chat-conversation-item-info-box"]);
        info_box.set_hexpand(true);
        root.append(&info_box);
        
        let title_label = gtk::Label::new(Some(&conversation.borrow().title));
        title_label.set_css_classes(&["ai-chat-conversation-item-title-label"]);
        title_label.set_hexpand(true);
        title_label.set_xalign(0.0);
        title_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        info_box.append(&title_label);

        let title_input = gtk::Entry::new();
        title_input.set_text(&conversation.borrow().title);
        title_input.set_css_classes(&["ai-chat-conversation-item-title-input"]);
        title_input.set_hexpand(true);
        title_input.set_visible(false);
        title_input.connect_activate(clone!(
            #[weak] title_label,
            #[weak] title_input,
            #[strong] conversation,
            move |_| {
                let new_title = title_input.text().to_string();
                if !new_title.is_empty() && new_title != conversation.borrow().title {
                    glib::spawn_future_local(clone!(
                        #[strong(rename_to = id)] conversation.borrow().id,
                        #[strong] new_title,
                        async move {
                            ai::conversation::rename_conversation(id, &new_title).await;
                        }
                    ));
                    conversation.borrow_mut().title = new_title;
                    title_input.set_visible(false);
                    title_label.set_visible(true);
                }
            }
        ));
        info_box.append(&title_input);

        let conversation_id = conversation.borrow().id;
        let message_count = aichats::get_messages_length(conversation_id).await.unwrap_or(0);
        let length_label = gtk::Label::new(Some(&message_count_str(message_count)));
        length_label.set_css_classes(&["ai-chat-conversation-item-length-label"]);
        length_label.set_halign(gtk::Align::Start);
        length_label.set_xalign(0.0);
        info_box.append(&length_label);

        let controls_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        controls_box.set_css_classes(&["ai-chat-conversation-item-controls-box"]);

        let rename_button = conversation_control_button("edit", "Rename Conversation");
        rename_button.connect_clicked(clone!(
            #[weak] title_label,
            #[weak] title_input,
            #[strong] conversation,
            move |_| {
                if WidgetExt::is_visible(&title_input) {
                    title_label.set_visible(true);
                    title_input.set_visible(false);
                } else {
                    title_label.set_visible(false);
                    title_input.set_visible(true);
                    title_input.set_text(&conversation.borrow().title);
                    title_input.grab_focus();
                }
            }
        ));
        controls_box.append(&rename_button);

        let delete_button = conversation_control_button("close", "Delete Conversation");
        delete_button.connect_clicked(clone!(
            #[strong] conversation,
            move |_| {
                glib::spawn_future_local(ai::conversation::delete_conversation(conversation.borrow().id));
            }
        ));
        controls_box.append(&delete_button);

        let controls_revealer = gtk::Revealer::new();
        controls_revealer.set_css_classes(&["ai-chat-conversation-item-controls-revealer"]);
        controls_revealer.set_valign(gtk::Align::Start);
        controls_revealer.set_transition_type(gtk::RevealerTransitionType::Crossfade);
        controls_revealer.set_transition_duration(200);
        controls_revealer.set_child(Some(&controls_box));
        root.append(&controls_revealer);

        root.add_controller(gesture::on_enter(clone!(
            #[weak] controls_revealer,
            move |_, _| {
                controls_revealer.set_reveal_child(true);
            }
        )));

        root.add_controller(gesture::on_leave(move || {
            controls_revealer.set_reveal_child(false);
        }));

        info_box.add_controller(gesture::on_primary_up(clone!(
            #[strong] conversation,
            move |_, _, _| if !WidgetExt::is_visible(&title_input) {
                glib::spawn_future_local(ai::conversation::load_conversation(conversation.borrow().id));
            }
        )));

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
        self.length_label.set_text(&message_count_str(new_length));
    }
}

#[derive(Debug, Clone, glib::Downgrade)]
pub struct ConversationsList {
    pub root: gtk::Box,
    pub conversations: Rc<RefCell<Vec<ConversationItem>>>,
}

impl ConversationsList {
    pub fn new() -> Self {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root.set_css_classes(&["ai-chat-conversations-list"]);

        let me = Self {
            root,
            conversations: Rc::new(RefCell::new(Vec::new())),
        };

        // Listen for events from the AI singleton channel
        let receiver = ai::CHANNEL.get().map(|channel| channel.subscribe());
        if let Some(mut receiver) = receiver {
            glib::spawn_future_local(clone!(
                #[weak] me,
                async move {
                    while let Ok(message) = receiver.recv().await {
                        match message {
                            AiChannelMessage::ConversationAdded(conversation) => {
                                let item = ConversationItem::new(conversation).await;
                                me.root.append(&item.root);
                                me.conversations.borrow_mut().push(item);
                            },

                            AiChannelMessage::ConversationDeleted(conversation_id) => {
                                let mut conversations = me.conversations.borrow_mut();
                                if let Some(pos) = conversations.iter().position(|item| item.conversation.borrow().id == conversation_id) {
                                    let item = conversations.remove(pos);
                                    me.root.remove(&item.root);
                                }
                            },

                            AiChannelMessage::ConversationRenamed(conversation_id, new_title) => {
                                let conversations = me.conversations.borrow();
                                for item in conversations.iter() {
                                    if item.conversation.borrow().id == conversation_id {
                                        item.set_title(&new_title);
                                        break;
                                    }
                                }
                            },

                            AiChannelMessage::ConversationTrimmed(conversation_id, _) => {
                                let conversations = me.conversations.borrow().clone();
                                for item in &conversations {
                                    if item.conversation.borrow().id == conversation_id {
                                        let current_length = aichats::get_messages_length(conversation_id).await.unwrap_or(0);
                                        item.set_length(current_length);
                                        break;
                                    }
                                }
                            },

                            AiChannelMessage::ConversationLoaded(conversation) => {
                                let conversations = me.conversations.borrow().clone();
                                for item in &conversations {
                                    if item.conversation.borrow().id == conversation.id {
                                        let current_length = aichats::get_messages_length(conversation.id).await.unwrap_or(0);
                                        item.set_length(current_length);
                                        break;
                                    }
                                }
                            },

                            AiChannelMessage::CycleStarted |
                            AiChannelMessage::CycleFinished => {
                                let Some(current_conversation_id) = ai::current_conversation_id() else {
                                    continue;
                                };

                                let conversations = me.conversations.borrow().clone();
                                for item in &conversations {
                                    if item.conversation.borrow().id == current_conversation_id {
                                        let current_length = aichats::get_messages_length(current_conversation_id).await.unwrap_or(0);
                                        item.set_length(current_length);
                                        break;
                                    }
                                }
                            }

                            _ => {},
                        }
                    }
                }
            ));
        }

        // Add existing conversations from the database
        glib::spawn_future_local({
            let root = me.root.clone();
            let conversations = me.conversations.clone();
            async move {
                if let Ok(existing_conversations) = aichats::get_all_conversations().await {
                    for conversation in existing_conversations {
                        let item = ConversationItem::new(conversation).await;
                        root.append(&item.root);
                        conversations.borrow_mut().push(item);
                    }
                }
            }
        });

        me
    }
}