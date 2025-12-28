mod chat;
mod conversations;

use std::time::Duration;
use gtk4::prelude::*;
use async_openai::types::chat::{
    ChatCompletionRequestMessage,
    ChatCompletionRequestUserMessageContent,
    ChatCompletionRequestAssistantMessageContent,
    ChatCompletionMessageToolCalls,
};

use crate::singletons::openai;

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
    conversation_title.set_halign(gtk4::Align::Start);
    conversation_title.set_xalign(0.0);
    conversation_title.set_hexpand(true);
    conversation_title.set_css_classes(&["ai-chat-conversation-title"]);
    conversation_controls.append(&conversation_title);

    let clear_conversation_button = conversation_control_button("clear_all", "Clear");
    clear_conversation_button.connect_clicked(move |_| {
        if !openai::is_currently_in_cycle() {
            let conversation_id = if let Some(session) = openai::SESSION.get()
                && let Some(conversation) = &*session.conversation.read().unwrap()
            {
                conversation.id
            } else {
                return;
            };

            openai::conversation::clear_conversation(conversation_id);
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

            let id = openai::send_user_message(&text);
            let message = chat::ChatMessage::new(
                &chat::ChatRole::User,
                Some(text),
            );
            message.set_id(id);
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
                                            ChatCompletionRequestUserMessageContent::Text(str) => Some(str.clone()),
                                            _ => Some("[Unsupported content]".to_owned()),
                                        },
                                    );
                                    message.set_id(*id);
                                    chat.add_message(message);
                                },

                                ChatCompletionRequestMessage::Assistant(msg) => {
                                    let message = chat::ChatMessage::new(
                                        &chat::ChatRole::Assistant,
                                        match &msg.content {
                                            Some(ChatCompletionRequestAssistantMessageContent::Text(str)) => Some(str.clone()),
                                            _ => Some("[Unsupported content]".to_owned()),
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
                            None,
                        ));
                    },

                    openai::AIChannelMessage::StreamChunk(chunk) => {
                        if let Some(latest_message) = chat.messages.borrow_mut().last_mut() {
                            let new_content = format!("{}{}", latest_message.content.as_deref().unwrap_or_default(), chunk);
                            latest_message.set_content(&new_content);
                        }
                    },

                    openai::AIChannelMessage::StreamComplete(id) => {
                        if let Some(latest_message) = chat.messages.borrow_mut().last_mut() {
                            latest_message.set_id(id);
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
        openai::conversation::add_conversation("Untitled");
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
    if let Some(channel) = openai::CHANNEL.get() {
        let mut receiver = channel.subscribe();

        gtk4::glib::spawn_future_local(async move {
            while let Ok(message) = receiver.recv().await {
                if let openai::AIChannelMessage::ConversationLoaded(_) = message {
                    ui_stack.set_visible_child_name("chat_ui");
                }
            }
        });
    }

    widget
}