use std::cell::{Cell, RefCell};
use std::rc::Rc;
use gtk4::prelude::*;

use crate::singletons::ai::{self, SESSION};
use super::chat::{Chat, ChatMessage, ChatRole};

const MIN_INPUT_HEIGHT: i32 = 50;
const MAX_INPUT_HEIGHT: i32 = 250;

pub struct ChatInput {
    pub widget: gtk4::Box,
    pub input_send_icon: gtk4::Label,
    pub input_send_label: gtk4::Label,
}

impl ChatInput {
    pub fn new(
        chat: &Chat,
        scroll_to_bottom: &Rc<dyn Fn()>,
    ) -> Self {
        let input_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        input_box.set_css_classes(&["ai-chat-input-box"]);

        let input_scrolled_window = gtk4::ScrolledWindow::new();
        input_scrolled_window.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Never);
        input_scrolled_window.set_propagate_natural_height(true);
        input_scrolled_window.set_min_content_height(MIN_INPUT_HEIGHT);
        input_scrolled_window.set_max_content_height(MAX_INPUT_HEIGHT);
        input_box.append(&input_scrolled_window);

        let input_overlay = gtk4::Overlay::new();
        input_scrolled_window.set_child(Some(&input_overlay));

        let input_placeholder = gtk4::Label::new(Some("Type your message here"));
        input_placeholder.set_css_classes(&["ai-chat-input-placeholder"]);
        input_placeholder.set_halign(gtk4::Align::Start);
        input_placeholder.set_valign(gtk4::Align::Start);
        input_placeholder.set_can_target(false);
        input_overlay.add_overlay(&input_placeholder);

        let input = gtk4::TextView::new();
        let send_current_input = Rc::new({
            let chat = chat.clone();
            let scroll_to_bottom = scroll_to_bottom.clone();
            let input = input.clone();
            move || {
                let buffer = input.buffer();
                let text = buffer.text(
                    &buffer.start_iter(),
                    &buffer.end_iter(),
                    false
                ).to_string();

                if ai::is_currently_in_cycle() {
                    if let Some(session) = SESSION.get()
                        && let Ok(mut stop_flag) = session.stop_cycle_flag.write()
                    {
                        *stop_flag = true;
                    }
                } else if !text.is_empty() {
                    let id = ai::send_user_message(&text);
                    let message = ChatMessage::new(
                        &ChatRole::User,
                        Some(text),
                    );
                    message.set_id(id);
                    chat.add_message(message);
                    scroll_to_bottom();

                    tokio::spawn(ai::start_request_cycle());

                    input.buffer().set_text("");
                }
            }
        });

        let measuring = Rc::new(Cell::new(false));
        input.set_wrap_mode(gtk4::WrapMode::WordChar);
        input.set_css_classes(&["ai-chat-input"]);
        input.set_hexpand(true);
        input.set_valign(gtk4::Align::Start);
        input.buffer().connect_changed({
            let input = input.clone();
            move |buffer| {
                if buffer.text(
                    &buffer.start_iter(),
                    &buffer.end_iter(),
                    false
                ).is_empty() {
                    input_placeholder.set_visible(true);
                } else {
                    input_placeholder.set_visible(false);
                }

                if measuring.get() {
                    return;
                }
                measuring.set(true);

                // we'll wait for the height to change, gtk4 is weird
                // TODO: move this logic into a gtk4 utils module, perhaps
                gtk4::glib::spawn_future_local({
                    let input = input.clone();
                    let input_scrolled_window = input_scrolled_window.clone();
                    let measuring = measuring.clone();
                    async move {
                        let old_height = input.allocation().height();
                        let (sender, receiver) = tokio::sync::oneshot::channel::<i32>();
                        let sender = Rc::new(RefCell::new(Some(sender)));

                        let tick_id = input.add_tick_callback({
                            let sender = sender.clone();
                            move |widget, _| {
                                let new_height = widget.allocation().height();
                                if new_height != old_height {
                                    if let Some(s) = sender.borrow_mut().take() {
                                        let _ = s.send(new_height);
                                    }
                                    gtk4::glib::ControlFlow::Break
                                } else {
                                    gtk4::glib::ControlFlow::Continue
                                }
                            }
                        });
                    
                        gtk4::glib::timeout_add_local_once(std::time::Duration::from_millis(500), {
                            let sender = sender.clone();
                            move || {
                                if let Some(s) = sender.borrow_mut().take() {
                                    let _ = s.send(old_height);
                                }
                                tick_id.remove();
                            }
                        });

                        let height = receiver.await.unwrap_or(old_height);
                    
                        input_scrolled_window.set_vscrollbar_policy(if height > MIN_INPUT_HEIGHT {
                            gtk4::PolicyType::Automatic
                        } else {
                            gtk4::PolicyType::Never
                        });
                    
                        if height <= MAX_INPUT_HEIGHT {
                            input_scrolled_window.vadjustment().set_value(0.0);
                        }

                        measuring.set(false);
                    }
                });
            }
        });
        let key_controller = gtk4::EventControllerKey::new();
        key_controller.connect_key_pressed({
            let send_current_input = send_current_input.clone();
            move |_, key, _, state| {
                if (key == gtk4::gdk::Key::Return || key == gtk4::gdk::Key::KP_Enter)
                    && !state.contains(gtk4::gdk::ModifierType::SHIFT_MASK)
                {
                    send_current_input();
                    gdk4::glib::Propagation::Stop
                } else {
                    gdk4::glib::Propagation::Proceed
                }
            }
        });
        input.add_controller(key_controller);
        input_overlay.set_child(Some(&input));

        let input_send_button = gtk4::Button::new();
        input_send_button.set_css_classes(&["ai-chat-input-send-button"]);
        input_send_button.set_halign(gtk4::Align::End);
        input_send_button.set_valign(gtk4::Align::Start);
        input_send_button.connect_clicked(move |_| send_current_input());
        input_box.append(&input_send_button);

        let input_send_button_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        input_send_button.set_child(Some(&input_send_button_box));

        let input_send_label = gtk4::Label::new(Some("send"));
        input_send_button_box.append(&input_send_label);

        let input_send_icon = gtk4::Label::new(Some("keyboard_return"));
        input_send_icon.set_css_classes(&["ai-chat-input-send-button-icon"]);
        input_send_button_box.append(&input_send_icon);

        Self {
            widget: input_box,
            input_send_icon,
            input_send_label,
        }
    }

    pub fn set_send_button_running(&self, running: bool) {
        if running {
            self.input_send_icon.set_label("progress_activity");
            self.input_send_icon.add_css_class("running");
            self.input_send_label.set_label("stop");
        } else {
            self.input_send_icon.set_label("keyboard_return");
            self.input_send_icon.remove_css_class("running");
            self.input_send_label.set_label("send");
        }
    }
}