mod chat;

use std::rc::Rc;
use std::cell::RefCell;
use gtk4::prelude::*;

use crate::singletons::openai;

pub fn new() -> gtk4::Box {
    let blocked = Rc::new(RefCell::new(false));
    let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    widget.set_css_classes(&["AiChat"]);

    let chat = chat::Chat::default();
    let chat_window = gtk4::ScrolledWindow::new();
    chat_window.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
    chat_window.set_vexpand(true);
    chat_window.set_hexpand(true);
    chat_window.set_child(Some(&chat.root));
    widget.append(&chat_window);

    let input = gtk4::Entry::new();
    input.set_placeholder_text(Some("Type your message here..."));
    input.set_css_classes(&["ai-chat-input"]);
    input.set_hexpand(true);
    input.connect_activate({
        let chat = chat.clone();
        let blocked = blocked.clone();
        move |entry| {
            let text = entry.text().to_string();
            if text.is_empty() || *blocked.borrow() {
                return;
            }

            chat.add_message(chat::ChatMessage::new(
                &chat::ChatRole::User,
                text.clone(),
            ));

            openai::send_user_message(&text);
            tokio::spawn(openai::start_request_cycle());

            entry.set_text("");
        }
    });

    if let Some(channel) = openai::CHANNEL.get() {
        let mut receiver = channel.subscribe();

        gtk4::glib::spawn_future_local({
            let input = input.clone();
            async move {
                while let Ok(message) = receiver.recv().await {
                    match message {
                        openai::AIChannelMessage::CycleStarted => {
                            input.set_editable(false);
                            *blocked.borrow_mut() = true;
                        },

                        openai::AIChannelMessage::CycleFinished => {
                            input.set_editable(true);
                            *blocked.borrow_mut() = false;
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

                        openai::AIChannelMessage::ToolCall(tool_name, arguments) => {
                            chat.append_tool_call_to_latest_message(&tool_name, &arguments);
                        },

                        _ => {},
                    }
                }
            }
        });
    }

    widget.append(&input);

    widget
}