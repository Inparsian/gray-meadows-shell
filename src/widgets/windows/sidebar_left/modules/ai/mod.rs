mod chat;
mod conversations;
mod input;

use std::rc::Rc;
use std::time::Duration;
use gtk::prelude::*;

use crate::services::ai::{self, SESSION, AiChannelMessage};
use crate::services::ai::types::{AiConversationDelta, AiConversationItemPayload};
use self::chat::Chat;
use self::chat::message::{ChatMessage, ChatRole};

pub fn conversation_control_button(icon: &str, label: &str) -> gtk::Button {
    let button = gtk::Button::new();
    button.set_css_classes(&["ai-chat-conversation-control-button"]);
    button.set_valign(gtk::Align::Start);
    button.set_halign(gtk::Align::End);

    let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    button_box.set_css_classes(&["ai-chat-conversation-control-button-box"]);
    button.set_child(Some(&button_box));

    let button_icon = gtk::Label::new(Some(icon));
    button_icon.set_css_classes(&["ai-chat-conversation-control-button-icon"]);
    button_box.append(&button_icon);

    let button_label = gtk::Label::new(Some(label));
    button_label.set_css_classes(&["ai-chat-conversation-control-button-label"]);
    button_box.append(&button_label);

    button
}

pub fn conversation_ui_header_button(icon: &str, label: &str) -> gtk::Button {
    let button = gtk::Button::new();
    button.set_css_classes(&["ai-chat-conversation-ui-header-button"]);
    button.set_valign(gtk::Align::Center);
    button.set_halign(gtk::Align::Center);

    let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    button_box.set_css_classes(&["ai-chat-conversation-ui-header-button-box"]);
    button.set_child(Some(&button_box));

    let button_icon = gtk::Label::new(Some(icon));
    button_icon.set_css_classes(&["ai-chat-conversation-ui-header-button-icon"]);
    button_box.append(&button_icon);

    let button_label = gtk::Label::new(Some(label));
    button_label.set_css_classes(&["ai-chat-conversation-ui-header-button-label"]);
    button_box.append(&button_label);

    button
}

pub fn chat_ui(stack: &gtk::Stack) -> gtk::Box {
    let widget = gtk::Box::new(gtk::Orientation::Vertical, 4);
    widget.set_css_classes(&["ai-chat-ui"]);

    let conversation_controls = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    conversation_controls.set_css_classes(&["ai-chat-conversation-controls"]);
    widget.append(&conversation_controls);

    let conversation_title = gtk::Label::new(Some("Untitled"));
    conversation_title.set_xalign(0.0);
    conversation_title.set_hexpand(true);
    conversation_title.set_ellipsize(gtk::pango::EllipsizeMode::End);
    conversation_title.set_css_classes(&["ai-chat-conversation-title"]);
    conversation_controls.append(&conversation_title);

    let clear_conversation_button = conversation_control_button("clear_all", "Clear");
    clear_conversation_button.connect_clicked(move |_| {
        if !ai::is_currently_in_cycle()
            && let Some(conversation_id) = ai::current_conversation_id()
        {
            glib::spawn_future_local(ai::conversation::clear_conversation(conversation_id));
        }
    });
    conversation_controls.append(&clear_conversation_button);

    let conversation_switch_button = conversation_control_button("menu_book", "Conversations");
    conversation_switch_button.connect_clicked(clone!(
        #[weak] stack,
        move |_| {
            stack.set_visible_child_name("conversations_ui");
        }
    ));
    conversation_controls.append(&conversation_switch_button);

    let chat = Chat::default();
    let chat_window = gtk::ScrolledWindow::new();
    chat_window.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    chat_window.set_vexpand(true);
    chat_window.set_hexpand(true);
    chat_window.set_child(Some(&chat.root));
    widget.append(&chat_window);

    let scroll_to_bottom: Rc<dyn Fn()> = Rc::new(move || {
        glib::timeout_add_local_once(Duration::from_millis(50), clone!(
            #[weak] chat_window,
            move || {
                let adjustment = chat_window.vadjustment();
                adjustment.set_value(adjustment.upper() - adjustment.page_size());
            }
        ));
    });

    let input = input::ChatInput::new(
        &chat,
        &scroll_to_bottom,
    );
    widget.append(&input.widget);

    if let Some(channel) = ai::CHANNEL.get() {
        let mut receiver = channel.subscribe();

        glib::spawn_future_local(async move {
            let chat = chat.clone();
            let conversation_title = conversation_title.clone();
            while let Ok(message) = receiver.recv().await {
                match message {
                    AiChannelMessage::ConversationLoaded(conversation) => {
                        let Some(session) = SESSION.get() else {
                            continue;
                        };

                        chat.clear_messages();
                        conversation_title.set_text(&conversation.title);

                        for item in session.items.read().unwrap().iter() {
                            match &item.payload {
                                AiConversationItemPayload::Message { role, content, .. } => match role.as_str() {
                                    "user" => chat.message_for(ChatRole::User, Some(item.id))
                                        .push_content(content, Some(item.id)),

                                    "assistant" => chat.message_for(ChatRole::Assistant, Some(item.id))
                                        .push_content(content, Some(item.id)),

                                    _ => {},
                                },

                                AiConversationItemPayload::Image { uuid } => {
                                    chat.message_for(ChatRole::User, Some(item.id)).push_image(uuid);
                                },

                                AiConversationItemPayload::Reasoning { summary, .. } => {
                                    let message = chat.message_for(ChatRole::Assistant, Some(item.id));

                                    // Don't even bother adding this thinking block if the summary is empty
                                    if !summary.is_empty() {
                                        message.push_thinking(summary);
                                    }
                                },

                                AiConversationItemPayload::FunctionCall { name, arguments, .. } => {
                                    chat.message_for(ChatRole::Assistant, Some(item.id)).push_tool_call(name, arguments);
                                },

                                AiConversationItemPayload::FunctionCallOutput { .. } => {
                                    chat.message_for(ChatRole::Assistant, Some(item.id));
                                },

                                AiConversationItemPayload::WebSearchCall { .. } => {
                                    chat.message_for(ChatRole::Assistant, Some(item.id)).push_web_search();
                                },

                                AiConversationItemPayload::ResponseBoundary => {
                                    chat.close_latest();
                                },
                            }
                        }
                    },

                    AiChannelMessage::ConversationTrimmed(conversation_id, down_to_message_id) => {
                        if ai::current_conversation_id() == Some(conversation_id) {
                            chat.trim_messages(down_to_message_id);
                        }
                    },

                    AiChannelMessage::ConversationRenamed(conversation_id, new_title) => {
                        if ai::current_conversation_id() == Some(conversation_id) {
                            conversation_title.set_text(&new_title);
                        }
                    },

                    AiChannelMessage::CycleStarted => {
                        input.set_send_button_running(true);
                    },

                    AiChannelMessage::CycleFailed | AiChannelMessage::CycleFinished => {
                        input.set_send_button_running(false);

                        // Items that were already written before a failure are kept in the database,
                        // so only drop the message if nothing made it into it
                        if chat.latest_message().is_some_and(|latest| latest.role == ChatRole::Assistant && latest.is_empty()) {
                            chat.remove_latest_message();
                        } else {
                            chat.close_latest();
                        }
                    },

                    AiChannelMessage::StreamStart => {
                        // Keep using the same message across tool call round trips
                        if !chat.latest_message().is_some_and(|latest| latest.role == ChatRole::Assistant && !latest.closed.get()) {
                            chat.add_message(ChatMessage::new_pending(ChatRole::Assistant));
                        }
                    },

                    AiChannelMessage::StreamMessageAdded => {
                        if let Some(latest_message) = chat.latest_message() {
                            latest_message.begin_content();
                        }
                    },

                    AiChannelMessage::StreamReasoningAdded => {
                        if let Some(latest_message) = chat.latest_message() {
                            latest_message.begin_thinking();
                        }
                    },

                    AiChannelMessage::StreamChunk(chunk) => if let Some(latest_message) = chat.latest_message() {
                        match chunk {
                            AiConversationDelta::Message(delta) => latest_message.append_content_delta(&delta),
                            AiConversationDelta::Reasoning(delta) => latest_message.append_reasoning_delta(&delta),
                        }
                    },

                    AiChannelMessage::StreamReasoningSummaryPartAdded => {
                        if let Some(latest_message) = chat.latest_message() {
                            latest_message.reasoning_part_added();
                        }
                    },

                    AiChannelMessage::StreamComplete(items) => {
                        if let Some(latest_message) = chat.latest_message() {
                            latest_message.assign_item_ids(&items);
                        }
                    },

                    AiChannelMessage::ToolCall(tool_name, arguments) => {
                        if let Some(latest_message) = chat.latest_message() {
                            latest_message.push_tool_call(&tool_name, &arguments);
                        }
                    },

                    AiChannelMessage::WebSearchCall => {
                        if let Some(latest_message) = chat.latest_message() {
                            latest_message.push_web_search();
                        }
                    },

                    _ => {},
                }

                scroll_to_bottom();
            }
        });
    }

    widget
}

pub fn conversations_ui(stack: &gtk::Stack) -> gtk::Box {
    let widget = gtk::Box::new(gtk::Orientation::Vertical, 8);
    widget.set_css_classes(&["ai-conversations-ui"]);

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    header.set_css_classes(&["ai-conversations-ui-header"]);
    widget.append(&header);

    let back_button = conversation_ui_header_button("arrow_back", "Back");
    back_button.connect_clicked(clone!(
        #[weak] stack,
        move |_| {
            stack.set_visible_child_name("chat_ui");
        }
    ));
    header.append(&back_button);

    let new_conversation_button = conversation_ui_header_button("add", "New Conversation");
    new_conversation_button.connect_clicked(move |_| {
        glib::spawn_future_local(ai::conversation::add_conversation("Untitled"));
    });
    header.append(&new_conversation_button);

    let conversations_list = conversations::ConversationsList::new();
    let conversations_window = gtk::ScrolledWindow::new();
    conversations_window.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    conversations_window.set_vexpand(true);
    conversations_window.set_hexpand(true);
    conversations_window.set_child(Some(&conversations_list.root));
    widget.append(&conversations_window);

    widget
}

pub fn new() -> gtk::Box {
    let widget = gtk::Box::new(gtk::Orientation::Vertical, 4);
    widget.set_css_classes(&["AiChat"]);

    let ui_stack = gtk::Stack::new();
    ui_stack.set_css_classes(&["ai-chat-ui-stack"]);
    ui_stack.set_vexpand(true);
    ui_stack.set_hexpand(true);
    ui_stack.set_transition_type(gtk::StackTransitionType::UnderRight);
    ui_stack.set_transition_duration(150);
    ui_stack.add_named(&chat_ui(&ui_stack), Some("chat_ui"));
    ui_stack.add_named(&conversations_ui(&ui_stack), Some("conversations_ui"));
    ui_stack.set_visible_child_name("chat_ui");

    widget.append(&ui_stack);

    // Go back when a new conversation is loaded
    if let Some(channel) = ai::CHANNEL.get() {
        let mut receiver = channel.subscribe();

        glib::spawn_future_local(async move {
            while let Ok(message) = receiver.recv().await {
                if let AiChannelMessage::ConversationLoaded(_) = message {
                    ui_stack.set_visible_child_name("chat_ui");
                }
            }
        });
    }

    widget
}