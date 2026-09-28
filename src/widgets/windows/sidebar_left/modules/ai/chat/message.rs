use std::rc::Rc;
use std::cell::{Cell, RefCell};
use std::path::Path;
use gtk::prelude::*;

use crate::USERNAME;
use crate::config::read_config;
use crate::services::ai;
use crate::services::ai::types::AiConversationItemPayload;
use crate::utils::{filesystem, gesture};
use crate::widgets::common::loading;
use super::elements::{self, ChatContentElement, ChatElement, ChatThinkingBlock};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ChatRole {
    User,
    Assistant
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub id: Rc<Cell<Option<i64>>>,
    pub last_item_id: Rc<Cell<Option<i64>>>,
    pub role: ChatRole,
    pub closed: Rc<Cell<bool>>,
    pub root: gtk::Box,
    pub elements: gtk::Box,
    element_list: Rc<RefCell<Vec<ChatElement>>>,
    loading: Rc<RefCell<Option<gtk::DrawingArea>>>,
    // Whether the next streamed delta of a given kind should start a new element
    pending_new_content: Rc<Cell<bool>>,
    pending_new_thinking: Rc<Cell<bool>>,
}

impl ChatMessage {
    fn default_assistant_icon() -> gtk::Widget {
        let sender_mui_icon = gtk::Label::new(Some("robot"));
        sender_mui_icon.set_css_classes(&["ai-chat-message-sender-mui-icon"]);
        sender_mui_icon.set_halign(gtk::Align::Start);
        sender_mui_icon.set_xalign(0.0);
        sender_mui_icon.upcast()
    }

    pub fn new(role: ChatRole) -> Self {
        let app_config = read_config();
        let id = Rc::new(Cell::new(None));
        let last_item_id: Rc<Cell<Option<i64>>> = Rc::new(Cell::new(None));
        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root.set_css_classes(&["ai-chat-message"]);
        root.set_valign(gtk::Align::Start);

        let top = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        top.set_css_classes(&["ai-chat-message-header"]);

        let sender_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        sender_box.set_css_classes(&["ai-chat-message-sender"]);

        let sender_icon: gtk::Widget = match role {
            ChatRole::User => {
                let face_path = format!("{}/.face", filesystem::get_home_directory());
                if Path::new(&face_path).exists() {
                    let sender_face = gtk::Image::new();
                    sender_face.set_css_classes(&["ai-chat-message-sender-icon"]);
                    sender_face.set_pixel_size(24);
                    sender_face.set_halign(gtk::Align::Start);
                    sender_face.set_from_file(Some(face_path));
                    sender_face.upcast()
                } else {
                    let sender_mui_icon = gtk::Label::new(Some("person"));
                    sender_mui_icon.set_css_classes(&["ai-chat-message-sender-mui-icon"]);
                    sender_mui_icon.set_halign(gtk::Align::Start);
                    sender_mui_icon.set_xalign(0.0);
                    sender_mui_icon.upcast()
                }
            },
            
            ChatRole::Assistant => app_config.ai.assistant_icon_path.as_ref().map_or_else(|| {
                Self::default_assistant_icon()
            }, |icon_path| if Path::new(icon_path).exists() {
                let assistant_icon = gtk::Image::new();
                assistant_icon.set_css_classes(&["ai-chat-message-sender-icon"]);
                assistant_icon.set_pixel_size(24);
                assistant_icon.set_halign(gtk::Align::Start);
                assistant_icon.set_from_file(Some(icon_path));
                assistant_icon.upcast()
            } else {
                Self::default_assistant_icon()
            }),
        };

        let sender_label = gtk::Label::new(Some(match role {
            ChatRole::User => &USERNAME,
            ChatRole::Assistant => app_config.ai.assistant_name.as_ref().map_or("AI Assistant", |name| name.as_str()),
        }));
        sender_label.set_css_classes(&["ai-chat-message-sender-label"]);
        sender_label.set_halign(gtk::Align::Start);
        sender_label.set_xalign(0.0);

        sender_box.append(&sender_icon);
        sender_box.append(&sender_label);
        top.append(&sender_box);

        let controls_revealer = gtk::Revealer::new();
        controls_revealer.set_css_classes(&["ai-chat-message-controls-revealer"]);
        controls_revealer.set_halign(gtk::Align::End);
        controls_revealer.set_valign(gtk::Align::Start);
        controls_revealer.set_hexpand(true);
        controls_revealer.set_transition_type(gtk::RevealerTransitionType::Crossfade);
        controls_revealer.set_transition_duration(150);

        let controls_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        controls_box.set_css_classes(&["ai-chat-message-controls-box"]);
        controls_revealer.set_child(Some(&controls_box));
        
        let delete_button = gtk::Button::new();
        delete_button.set_css_classes(&["ai-chat-message-control-button"]);
        delete_button.set_label("delete");
        delete_button.connect_clicked(clone!(
            #[strong] id,
            move |_| if !ai::is_currently_in_cycle() && let Some(message_id) = id.get() {
                glib::spawn_future_local(ai::trim_items(message_id));
            }
        ));
        controls_box.append(&delete_button);

        let element_list: Rc<RefCell<Vec<ChatElement>>> = Rc::new(RefCell::new(Vec::new()));

        let edit_button = gtk::Button::new();
        edit_button.set_css_classes(&["ai-chat-message-control-button"]);
        edit_button.set_label("edit");
        edit_button.connect_clicked(clone!(
            #[strong] element_list,
            move |_| if !ai::is_currently_in_cycle() {
                for element in element_list.borrow().iter() {
                    if let ChatElement::Content(content) = element {
                        content.start_editing();
                    }
                }
            }
        ));
        controls_box.append(&edit_button);

        let retry_button = gtk::Button::new();
        retry_button.set_css_classes(&["ai-chat-message-control-button"]);
        retry_button.set_label("refresh");
        retry_button.connect_clicked(clone!(
            #[strong] id,
            #[strong] last_item_id,
            #[strong] role,
            move |_| if !ai::is_currently_in_cycle() {
                // For user messages, trim down to the assistant response directly after it
                let trim_to = match role {
                    ChatRole::User => last_item_id.get().map(|last_id| last_id + 1),
                    ChatRole::Assistant => id.get(),
                };

                if let Some(trim_to) = trim_to {
                    tokio::spawn(async move {
                        ai::trim_items(trim_to).await;
                        ai::start_request_cycle().await;
                    });
                }
            }
        ));
        controls_box.append(&retry_button);

        top.append(&controls_revealer);

        // Every item of this message lives here, in the order they are in inside of the database
        let elements = gtk::Box::new(gtk::Orientation::Vertical, 12);
        elements.set_css_classes(&["ai-chat-message-elements"]);
        elements.set_valign(gtk::Align::Start);

        root.append(&top);
        root.append(&elements);

        root.add_controller(gesture::on_enter(clone!(
            #[weak] controls_revealer,
            move |_, _| {
                controls_revealer.set_reveal_child(true);
            }
        )));

        root.add_controller(gesture::on_leave(move || {
            controls_revealer.set_reveal_child(false);
        }));

        Self {
            id,
            last_item_id,
            role,
            closed: Rc::new(Cell::new(false)),
            root,
            elements,
            element_list,
            loading: Rc::new(RefCell::new(None)),
            pending_new_content: Rc::new(Cell::new(false)),
            pending_new_thinking: Rc::new(Cell::new(false)),
        }
    }

    // Creates a message that shows a loading indicator until its first element is pushed
    pub fn new_pending(role: ChatRole) -> Self {
        let message = Self::new(role);
        let loading = loading::new();
        loading.set_halign(gtk::Align::Start);
        loading.set_valign(gtk::Align::Start);
        message.elements.append(&loading);
        message.loading.replace(Some(loading));
        message
    }

    pub fn record_item(&self, id: i64) {
        if self.id.get().is_none() {
            self.id.set(Some(id));
        }

        if self.last_item_id.get().is_none_or(|last_id| id > last_id) {
            self.last_item_id.set(Some(id));
        }
    }

    pub fn is_empty(&self) -> bool {
        self.element_list.borrow().is_empty()
    }

    fn push(&self, element: ChatElement) {
        if let Some(loading) = self.loading.take() {
            self.elements.remove(&loading);
        }

        self.elements.append(&element.widget());
        self.element_list.borrow_mut().push(element);
    }

    pub fn push_content(&self, content: &str, item_id: Option<i64>) {
        self.push(ChatElement::Content(ChatContentElement::new(content, item_id)));
    }

    pub fn push_thinking(&self, summary: &str) {
        let thinking_block = ChatThinkingBlock::new();
        thinking_block.set_summary(summary);
        self.push(ChatElement::Thinking(thinking_block));
    }

    pub fn push_tool_call(&self, tool_name: &str, arguments: &str) {
        self.push(ChatElement::ToolCall(elements::tool_call(tool_name, arguments)));
    }

    pub fn push_web_search(&self) {
        self.push(ChatElement::WebSearch(elements::web_search_call()));
    }

    pub fn push_image(&self, uuid: &str) {
        if let Some(image) = elements::image(uuid) {
            self.push(ChatElement::Image(image));
        }
    }

    // The next content delta will start a new content element
    pub fn begin_content(&self) {
        self.pending_new_content.set(true);
    }

    // The next reasoning delta will start a new thinking block
    pub fn begin_thinking(&self) {
        self.pending_new_thinking.set(true);
    }

    pub fn append_content_delta(&self, delta: &str) {
        if !self.pending_new_content.replace(false)
            && let Some(ChatElement::Content(content)) = self.element_list.borrow().last()
        {
            content.append_content(delta);
            return;
        }

        self.push_content(delta, None);
    }

    pub fn append_reasoning_delta(&self, delta: &str) {
        if !self.pending_new_thinking.replace(false)
            && let Some(ChatElement::Thinking(thinking)) = self.element_list.borrow().last()
        {
            thinking.append_summary(delta);
            return;
        }

        self.push_thinking(delta);
    }

    pub fn reasoning_part_added(&self) {
        if let Some(ChatElement::Thinking(thinking)) = self.element_list.borrow().last() {
            thinking.new_part();
        }
    }

    // Assigns database IDs to items that were streamed in before they were written
    pub fn assign_item_ids(&self, items: &[(i64, AiConversationItemPayload)]) {
        let element_list = self.element_list.borrow();
        let mut unassigned_contents = element_list.iter().filter_map(|element| match element {
            ChatElement::Content(content) if content.item_id.get().is_none() => Some(content),
            _ => None,
        });

        for (id, payload) in items {
            self.record_item(*id);

            if matches!(payload, AiConversationItemPayload::Message { .. })
                && let Some(content) = unassigned_contents.next()
            {
                content.item_id.set(Some(*id));
            }
        }
    }
}
