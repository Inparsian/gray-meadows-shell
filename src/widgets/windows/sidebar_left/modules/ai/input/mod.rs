mod attachments;

use std::rc::Rc;
use gtk4::prelude::*;

use crate::singletons::ai::{self, SESSION};
use crate::singletons::ai::images::cache_image_data;
use crate::widgets::windows;
use crate::utils::allocation_watcher::{AllocationWatcher, AllocationWatcherOptions};
use super::chat::{Chat, ChatMessage, ChatRole};
use self::attachments::ImageAttachments;

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
        let input_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        input_box.set_css_classes(&["ai-chat-input-box"]);

        let input_attachments = ImageAttachments::default();
        input_box.append(&input_attachments.container);

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
        let input_watcher = AllocationWatcher::new(&input, AllocationWatcherOptions {
            timeout_millis: 250,
            max_allocation_width: None,
            max_allocation_height: None,
            min_allocation_width: 0,
            min_allocation_height: 0,
        });
        
        let send_current_input = {
            let chat = chat.clone();
            let scroll_to_bottom = scroll_to_bottom.clone();
            let input = input.downgrade();
            let input_attachments = input_attachments.clone();
            move || {
                let chat = chat.clone();
                let scroll_to_bottom = scroll_to_bottom.clone();
                let input = input.clone();
                let input_attachments = input_attachments.clone();
                async move {
                    let Some(input) = input.upgrade() else {
                        return;
                    };
                    
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
                    } else if input_attachments.get_attachments().is_empty() || input_attachments.all_ready() {
                        #[allow(clippy::if_then_some_else_none)]
                        let text_sent = if !text.is_empty() {
                            let id = ai::send_user_message(&text).await;
                            let message = ChatMessage::new(
                                ChatRole::User,
                                Some(text),
                            );
                            message.set_id(id);
                            chat.add_message(message);
    
                            input.buffer().set_text("");
                            Some(id)
                        } else {
                            None
                        };
    
                        let ready_attachments = input_attachments.get_attachments()
                            .into_iter()
                            .flatten()
                            .collect::<Vec<_>>();
    
                        for attachment in &ready_attachments {
                            if let Ok(path) = cache_image_data(&attachment.base64) {
                                let id = ai::send_user_image(&path).await;
                                chat.assert_last_message_is_role(ChatRole::User, text_sent.or(Some(id)));
                                chat.append_image_to_latest_message(&path);
                            }
                        }
    
                        input_attachments.clear();
    
                        if (text_sent.is_some() && input_attachments.get_attachments().is_empty()) || !ready_attachments.is_empty() {
                            scroll_to_bottom();
    
                            tokio::spawn(ai::start_request_cycle());
                        }
                    }
                }
            }
        };

        input.set_wrap_mode(gtk4::WrapMode::WordChar);
        input.set_css_classes(&["ai-chat-input"]);
        input.set_hexpand(true);
        input.set_valign(gtk4::Align::Start);
        input.buffer().connect_changed(move |buffer| {
            if buffer.text(
                &buffer.start_iter(),
                &buffer.end_iter(),
                false
            ).is_empty() {
                input_placeholder.set_visible(true);
            } else {
                input_placeholder.set_visible(false);
            }

            input_watcher.one_shot_future(clone!(
                #[strong(rename_to = last_received_allocation)] input_watcher.last_received_allocation,
                #[weak] input_scrolled_window,
                async move {
                    let height = last_received_allocation.get()
                        .map_or(MIN_INPUT_HEIGHT, |alloc| alloc.height());
                
                    input_scrolled_window.set_vscrollbar_policy(if height > MIN_INPUT_HEIGHT {
                        gtk4::PolicyType::Automatic
                    } else {
                        gtk4::PolicyType::Never
                    });
                
                    if height <= MAX_INPUT_HEIGHT {
                        input_scrolled_window.vadjustment().set_value(0.0);
                    }
                }
            ));
        });
        let key_controller = gtk4::EventControllerKey::new();
        key_controller.connect_key_pressed(clone!(
            #[strong] send_current_input,
            move |_, key, _, state| {
                if (key == gtk4::gdk::Key::Return || key == gtk4::gdk::Key::KP_Enter)
                    && !state.contains(gtk4::gdk::ModifierType::SHIFT_MASK)
                {
                    glib::spawn_future_local(send_current_input());
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
        ));
        input.add_controller(key_controller);
        input.connect_paste_clipboard(clone!(
            #[weak] input_attachments,
            move |input| {
                let clipboard = input.clipboard();
                let formats = clipboard.formats();
                
                if formats.contains_type(gdk4::Texture::static_type()) {
                    clipboard.read_texture_async(None::<&gtk4::gio::Cancellable>, move |result| {
                        match result {
                            Ok(Some(texture)) => {
                                input_attachments.push_texture(&texture);
                            },

                            Ok(None) => {
                                warn!("No texture found in clipboard");
                            },

                            Err(err) => {
                                warn!(?err, "Error reading texture from clipboard");
                            },
                        }
                    });
                }
            }
        ));

        input_overlay.set_child(Some(&input));

        let input_controls_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        input_controls_box.set_css_classes(&["ai-chat-input-controls-box"]);
        input_box.append(&input_controls_box);

        let input_attach_image_button = gtk4::Button::new();
        input_attach_image_button.set_css_classes(&["ai-chat-input-attach-image-button"]);
        input_attach_image_button.set_halign(gtk4::Align::Start);
        input_attach_image_button.set_valign(gtk4::Align::Start);
        input_attach_image_button.set_label("image");
        input_attach_image_button.connect_clicked(move |_| {
            let file_chooser = gtk4::FileChooserNative::new(
                Some("Select Image"),
                None::<&gtk4::Window>,
                gtk4::FileChooserAction::Open,
                Some("Open"),
                Some("Cancel"),
            );

            let filter = gtk4::FileFilter::new();
            filter.add_mime_type("image/png");
            filter.add_mime_type("image/jpeg");
            filter.set_name(Some("Image Files"));
            file_chooser.add_filter(&filter);

            file_chooser.connect_response(clone!(
                #[weak] input_attachments,
                move |file_chooser, response| {
                    if response == gtk4::ResponseType::Accept
                        && let Some(file) = file_chooser.file()
                        && let Ok(texture) = gdk4::Texture::from_file(&file)
                    {
                        input_attachments.push_texture(&texture);
                    }

                    windows::show("left_sidebar");
                }
            ));

            windows::hide("left_sidebar");
            file_chooser.show();
        });
        input_controls_box.append(&input_attach_image_button);

        let input_send_button = gtk4::Button::new();
        input_send_button.set_css_classes(&["ai-chat-input-send-button"]);
        input_send_button.set_halign(gtk4::Align::End);
        input_send_button.set_hexpand(true);
        input_send_button.set_valign(gtk4::Align::Start);
        input_send_button.connect_clicked(move |_| {
            glib::spawn_future_local(send_current_input());
        });
        input_controls_box.append(&input_send_button);

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