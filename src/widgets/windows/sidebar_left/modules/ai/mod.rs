mod chat;
mod conversations;
mod input;

use std::rc::Rc;
use std::time::Duration;
use gtk4::prelude::*;

use crate::singletons::ai::{self, SESSION, AiChannelMessage};
use crate::singletons::ai::types::{AiConversationDelta, AiConversationItem, AiConversationItemPayload};

pub fn conversation_control_button(icon: &str, label: &str) -> gtk4::Button {
    let button = gtk4::Button::new();
    button.set_css_classes(&["ai-chat-conversation-control-button"]);
    button.set_valign(gtk4::Align::Start);
    button.set_halign(gtk4::Align::End);

    let button_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    button_box.set_css_classes(&["ai-chat-conversation-control-button-box"]);
    button.set_child(Some(&button_box));

    let button_icon = gtk4::Label::new(Some(icon));
    button_icon.set_css_classes(&["ai-chat-conversation-control-button-icon"]);
    button_box.append(&button_icon);

    let button_label = gtk4::Label::new(Some(label));
    button_label.set_css_classes(&["ai-chat-conversation-control-button-label"]);
    button_box.append(&button_label);

    button
}

pub fn conversation_ui_header_button(icon: &str, label: &str) -> gtk4::Button {
    let button = gtk4::Button::new();
    button.set_css_classes(&["ai-chat-conversation-ui-header-button"]);
    button.set_valign(gtk4::Align::Center);
    button.set_halign(gtk4::Align::Center);

    let button_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    button_box.set_css_classes(&["ai-chat-conversation-ui-header-button-box"]);
    button.set_child(Some(&button_box));

    let button_icon = gtk4::Label::new(Some(icon));
    button_icon.set_css_classes(&["ai-chat-conversation-ui-header-button-icon"]);
    button_box.append(&button_icon);

    let button_label = gtk4::Label::new(Some(label));
    button_label.set_css_classes(&["ai-chat-conversation-ui-header-button-label"]);
    button_box.append(&button_label);

    button
}

pub fn chat_ui(stack: &gtk4::Stack) -> gtk4::Box {
    let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    widget.set_css_classes(&["ai-chat-ui"]);

    let conversation_controls = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    conversation_controls.set_css_classes(&["ai-chat-conversation-controls"]);
    widget.append(&conversation_controls);

    let conversation_title = gtk4::Label::new(Some("Untitled"));
    conversation_title.set_xalign(0.0);
    conversation_title.set_hexpand(true);
    conversation_title.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    conversation_title.set_css_classes(&["ai-chat-conversation-title"]);
    conversation_controls.append(&conversation_title);

    let clear_conversation_button = conversation_control_button("clear_all", "Clear");
    clear_conversation_button.connect_clicked(move |_| {
        if !ai::is_currently_in_cycle()
            && let Some(conversation_id) = ai::current_conversation_id()
        {
            ai::conversation::clear_conversation(conversation_id);
        }
    });
    conversation_controls.append(&clear_conversation_button);

    let conversation_switch_button = conversation_control_button("menu_book", "Conversations");
    conversation_switch_button.connect_clicked({
        let stack = stack.clone();
        move |_| {
            stack.set_visible_child_name("conversations_ui");
        }
    });
    conversation_controls.append(&conversation_switch_button);

    let chat = chat::Chat::default();
    let chat_window = gtk4::ScrolledWindow::new();
    chat_window.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
    chat_window.set_vexpand(true);
    chat_window.set_hexpand(true);
    chat_window.set_child(Some(&chat.root));
    widget.append(&chat_window);

    let scroll_to_bottom: Rc<dyn Fn()> = Rc::new(move || {
        gtk4::glib::timeout_add_local_once(Duration::from_millis(50), {
            let chat_window = chat_window.clone();
            move || {
                let adjustment = chat_window.vadjustment();
                adjustment.set_value(adjustment.upper() - adjustment.page_size());
            }
        });
    });

    let input = input::ChatInput::new(
        &chat,
        &scroll_to_bottom,
    );
    widget.append(&input.widget);

    if let Some(channel) = ai::CHANNEL.get() {
        let mut receiver = channel.subscribe();

        gtk4::glib::spawn_future_local(async move {
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

                        let mut processed_reasoning = false;
                        for (index, item) in session.items.read().unwrap().iter().enumerate() {
                            let assert_assistant_last_message = |chat: &chat::Chat, item: &AiConversationItem| {
                                if index == 0 || !matches!(
                                    session.items.read().unwrap().get(index - 1)
                                        .map(|it| match &it.payload {
                                            AiConversationItemPayload::Message { role, .. } => role.as_str(),
                                            _ => "",
                                        }),
                                    Some("assistant"),
                                ) {
                                    let message = chat::ChatMessage::new(
                                        &chat::ChatRole::Assistant,
                                        None,
                                    );
                                    message.set_id(item.id);
                                    chat.add_message(message);
                                }
                            };

                            match &item.payload {
                                AiConversationItemPayload::Message { role, content, .. } 
                                if matches!(role.as_str(), "user" | "assistant") => {
                                    // If we processed a reasoning payload before this one, modify the existing
                                    // assistant message instead of adding a new one
                                    if processed_reasoning {
                                        if let Some(latest_message) = chat.messages.borrow_mut().last_mut()
                                            && matches!(role.as_str(), "assistant")
                                        {
                                            latest_message.set_content(content);
                                        }
                                        processed_reasoning = false;
                                    } else {
                                        let message = chat::ChatMessage::new(
                                            match role.as_str() {
                                                "user" => &chat::ChatRole::User,
                                                "assistant" => &chat::ChatRole::Assistant,
                                                _ => unreachable!(),
                                            },
                                            Some(content.clone()),
                                        );

                                        message.set_id(item.id);
                                        chat.add_message(message);
                                    }
                                },

                                AiConversationItemPayload::Reasoning { summary, .. } => {
                                    assert_assistant_last_message(&chat, item);
                                    chat.append_thinking_block_to_latest_message(summary);

                                    // This DOES have to come before assistant messages, so we use this to
                                    // indicate that a new one should not be added if we encounter an assistant
                                    // message next.
                                    processed_reasoning = true;
                                },

                                AiConversationItemPayload::FunctionCall { name, arguments, .. } => {
                                    assert_assistant_last_message(&chat, item);
                                    chat.append_tool_call_to_latest_message(name, arguments);
                                },

                                _ => {},
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

                    AiChannelMessage::CycleFailed => {
                        input.set_send_button_running(false);
                        chat.remove_latest_message();
                    },

                    AiChannelMessage::CycleFinished => {
                        input.set_send_button_running(false);

                        if chat.messages.borrow().last().is_some_and(
                            |latest| latest.content.is_none() && latest.thinking.is_none()
                        ) {
                            chat.remove_latest_message();
                        }
                    },

                    AiChannelMessage::StreamStart => {
                        chat.add_message(chat::ChatMessage::new(
                            &chat::ChatRole::Assistant,
                            None,
                        ));
                    },

                    AiChannelMessage::StreamChunk(chunk) => {
                        match chunk {
                            AiConversationDelta::Message(delta) => {
                                if let Some(latest_message) = chat.messages.borrow_mut().last_mut() {
                                    let new_content = format!("{}{}", latest_message.content.as_deref().unwrap_or_default(), delta);
                                    latest_message.set_content(&new_content);
                                }
                            },

                            AiConversationDelta::Reasoning(delta) => {
                                if let Some(latest_message) = chat.messages.borrow_mut().last_mut() {
                                    let new_content = if let Some(thinking) = &mut latest_message.thinking {
                                        let current_summary = thinking.summary.as_deref().unwrap_or_default();
                                        format!("{}{}", current_summary, delta)
                                    } else {
                                        delta
                                    };

                                    latest_message.set_thinking(&new_content);
                                }
                            },
                        }
                    },

                    AiChannelMessage::StreamReasoningSummaryPartAdded => {
                        if let Some(latest_message) = chat.messages.borrow_mut().last_mut()
                            && let Some(thinking) = &mut latest_message.thinking
                        {
                            let current_summary = thinking.summary.as_deref().unwrap_or_default();
                            let new_content = if current_summary.is_empty() {
                                String::new()
                            } else {
                                format!("{}\n\n", current_summary)
                            };
                            thinking.set_summary(&new_content);
                        }
                    },

                    AiChannelMessage::StreamComplete(id) => {
                        if let Some(latest_message) = chat.messages.borrow_mut().last_mut() {
                            latest_message.set_id(id);
                        }
                    },

                    AiChannelMessage::ToolCall(tool_name, arguments) => {
                        chat.append_tool_call_to_latest_message(&tool_name, &arguments);
                    },

                    _ => {},
                }

                scroll_to_bottom();
            }
        });
    }

    widget
}

pub fn conversations_ui(stack: &gtk4::Stack) -> gtk4::Box {
    let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    widget.set_css_classes(&["ai-conversations-ui"]);

    let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    header.set_css_classes(&["ai-conversations-ui-header"]);
    widget.append(&header);

    let back_button = conversation_ui_header_button("arrow_back", "Back");
    back_button.connect_clicked({
        let stack = stack.clone();
        move |_| {
            stack.set_visible_child_name("chat_ui");
        }
    });
    header.append(&back_button);

    let new_conversation_button = conversation_ui_header_button("add", "New Conversation");
    new_conversation_button.connect_clicked(move |_| {
        ai::conversation::add_conversation("Untitled");
    });
    header.append(&new_conversation_button);

    let conversations_list = conversations::ConversationsList::new();
    let conversations_window = gtk4::ScrolledWindow::new();
    conversations_window.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
    conversations_window.set_vexpand(true);
    conversations_window.set_hexpand(true);
    conversations_window.set_child(Some(&conversations_list.root));
    widget.append(&conversations_window);

    widget
}

pub fn new() -> gtk4::Box {
    let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    widget.set_css_classes(&["AiChat"]);

    let ui_stack = gtk4::Stack::new();
    ui_stack.set_css_classes(&["ai-chat-ui-stack"]);
    ui_stack.set_vexpand(true);
    ui_stack.set_hexpand(true);
    ui_stack.set_transition_type(gtk4::StackTransitionType::UnderRight);
    ui_stack.set_transition_duration(150);
    ui_stack.add_named(&chat_ui(&ui_stack), Some("chat_ui"));
    ui_stack.add_named(&conversations_ui(&ui_stack), Some("conversations_ui"));
    ui_stack.set_visible_child_name("chat_ui");

    widget.append(&ui_stack);

    // Go back when a new conversation is loaded
    if let Some(channel) = ai::CHANNEL.get() {
        let mut receiver = channel.subscribe();

        gtk4::glib::spawn_future_local(async move {
            while let Ok(message) = receiver.recv().await {
                if let AiChannelMessage::ConversationLoaded(_) = message {
                    ui_stack.set_visible_child_name("chat_ui");
                }
            }
        });
    }

    widget
}