mod chat;

use std::time::Duration;
use gtk4::prelude::*;
use async_openai::types::chat::{
    ChatCompletionRequestMessage,
    ChatCompletionRequestUserMessageContent,
    ChatCompletionRequestAssistantMessageContent,
    ChatCompletionMessageToolCalls,
};

use crate::singletons::openai;

pub fn new() -> gtk4::Box {
    let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    widget.set_css_classes(&["AiChat"]);

    let conversation_controls = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    conversation_controls.set_css_classes(&["ai-chat-conversation-controls"]);
    widget.append(&conversation_controls);

    let conversation_title = gtk4::Label::new(Some("Untitled"));
    conversation_title.set_halign(gtk4::Align::Start);
    conversation_title.set_xalign(0.0);
    conversation_title.set_css_classes(&["ai-chat-conversation-title"]);
    conversation_controls.append(&conversation_title);

    let chat = chat::Chat::default();
    let chat_window = gtk4::ScrolledWindow::new();
    chat_window.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
    chat_window.set_vexpand(true);
    chat_window.set_hexpand(true);
    chat_window.set_child(Some(&chat.root));
    widget.append(&chat_window);

    let scroll_to_bottom = move || {
        gtk4::glib::timeout_add_local_once(Duration::from_millis(50), {
            let chat_window = chat_window.clone();
            move || {
                let adjustment = chat_window.vadjustment();
                adjustment.set_value(adjustment.upper() - adjustment.page_size());
            }
        });
    };

    let input = gtk4::Entry::new();
    input.set_placeholder_text(Some("Type your message here..."));
    input.set_css_classes(&["ai-chat-input"]);
    input.set_hexpand(true);
    input.connect_activate({
        let chat = chat.clone();
        let scroll_to_bottom = scroll_to_bottom.clone();
        move |entry| {
            let text = entry.text().to_string();
            if text.is_empty() || openai::is_currently_in_cycle() {
                return;
            }

            openai::send_user_message(&text);
            let message = chat::ChatMessage::new(
                &chat::ChatRole::User,
                text,
            );
            message.set_id(openai::get_highest_indice() + 1);
            chat.add_message(message);
            scroll_to_bottom();

            tokio::spawn(openai::start_request_cycle());

            entry.set_text("");
        }
    });

    if let Some(channel) = openai::CHANNEL.get() {
        let mut receiver = channel.subscribe();

        gtk4::glib::spawn_future_local(async move {
            let chat = chat.clone();
            let conversation_title = conversation_title.clone();
            while let Ok(message) = receiver.recv().await {
                match message {
                    openai::AIChannelMessage::ConversationLoaded(conversation) => {
                        chat.clear_messages();
                        conversation_title.set_text(&conversation.title);

                        for (id, msg) in &openai::get_sorted_messages() {
                            match msg {
                                ChatCompletionRequestMessage::User(msg) => {
                                    let message = chat::ChatMessage::new(
                                        &chat::ChatRole::User,
                                        match &msg.content {
                                            ChatCompletionRequestUserMessageContent::Text(str) => str.clone(),
                                            _ => String::new(),
                                        },
                                    );
                                    message.set_id(*id);
                                    chat.add_message(message);
                                },

                                ChatCompletionRequestMessage::Assistant(msg) => {
                                    let message = chat::ChatMessage::new(
                                        &chat::ChatRole::Assistant,
                                        match &msg.content {
                                            Some(ChatCompletionRequestAssistantMessageContent::Text(str)) => str.clone(),
                                            _ => String::new(),
                                        },
                                    );
                                    message.set_id(*id);
                                    chat.add_message(message);

                                    // Append tool call info if present
                                    if let Some(tool_calls) = &msg.tool_calls {
                                        for tool_call in tool_calls {
                                            if let ChatCompletionMessageToolCalls::Function(tool) = tool_call {
                                                chat.append_tool_call_to_latest_message(
                                                    &tool.function.name,
                                                    &tool.function.arguments,
                                                );
                                            }
                                        }
                                    }
                                },

                                // System, tool, etc. messages are disregarded for chat display
                                _ => {},
                            }
                        }
                    },

                    openai::AIChannelMessage::ConversationTrimmed(down_to_message_id) => {
                        chat.trim_messages(down_to_message_id);
                    },

                    openai::AIChannelMessage::StreamStart => {
                        chat.add_message(chat::ChatMessage::new(
                            &chat::ChatRole::Assistant,
                            String::new()
                        ));
                    },

                    openai::AIChannelMessage::StreamChunk(chunk) => {
                        if let Some(latest_message) = chat.messages.borrow_mut().last_mut() {
                            let new_content = format!("{}{}", latest_message.content, chunk);
                            latest_message.set_content(&new_content);
                        }
                    },

                    openai::AIChannelMessage::StreamComplete => {
                        if let Some(latest_message) = chat.messages.borrow_mut().last_mut() {
                            latest_message.set_id(openai::get_highest_indice() + 1);
                        }
                    },

                    openai::AIChannelMessage::ToolCall(tool_name, arguments) => {
                        chat.append_tool_call_to_latest_message(&tool_name, &arguments);
                    },

                    _ => {},
                }

                scroll_to_bottom();
            }
        });
    }

    widget.append(&input);

    widget
}