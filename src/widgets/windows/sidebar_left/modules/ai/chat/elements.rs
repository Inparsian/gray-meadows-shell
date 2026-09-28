use std::rc::Rc;
use std::cell::{Cell, RefCell};
use gtk::prelude::*;

use crate::services::ai;
use crate::services::ai::images::uuid_to_file_path;
use crate::services::ai::types::AiConversationItemPayload;
use crate::widgets::common::revealer::{AdwRevealer, AdwRevealerDirection, GEasing};
use super::content::ChatMessageContent;

#[derive(Debug, Clone)]
pub enum ChatElement {
    Content(ChatContentElement),
    Thinking(ChatThinkingBlock),
    ToolCall(gtk::Box),
    WebSearch(gtk::Box),
    Image(gtk::Widget),
}

impl ChatElement {
    pub fn widget(&self) -> gtk::Widget {
        match self {
            Self::Content(content) => content.view.clone().upcast(),
            Self::Thinking(thinking) => thinking.root.clone().upcast(),
            Self::ToolCall(bx) | Self::WebSearch(bx) => bx.clone().upcast(),
            Self::Image(widget) => widget.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChatThinkingBlock {
    pub root: gtk::Box,
    pub summary_root: gtk4cmark::MarkdownView,
    pub summary: Rc<RefCell<String>>,
}

impl ChatThinkingBlock {
    pub fn new() -> Self {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root.set_css_classes(&["ai-chat-thinking-block"]);

        let thinking_dropdown_button = gtk::Button::new();
        thinking_dropdown_button.set_css_classes(&["ai-chat-thinking-dropdown-button"]);
        thinking_dropdown_button.set_valign(gtk::Align::Start);
        thinking_dropdown_button.set_hexpand(true);
        root.append(&thinking_dropdown_button);

        let thinking_dropdown_header = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        thinking_dropdown_header.set_css_classes(&["ai-chat-thinking-dropdown-header"]);
        thinking_dropdown_header.set_hexpand(true);
        thinking_dropdown_button.set_child(Some(&thinking_dropdown_header));

        let thinking_dropdown_indicator = gtk::Label::new(Some("lightbulb_2"));
        thinking_dropdown_indicator.set_css_classes(&["ai-chat-thinking-dropdown-indicator"]);
        thinking_dropdown_indicator.set_halign(gtk::Align::Start);
        thinking_dropdown_indicator.set_xalign(0.0);
        thinking_dropdown_header.append(&thinking_dropdown_indicator);

        let thinking_dropdown_label = gtk::Label::new(Some("Thoughts"));
        thinking_dropdown_label.set_css_classes(&["ai-chat-thinking-dropdown-label"]);
        thinking_dropdown_label.set_halign(gtk::Align::Start);
        thinking_dropdown_label.set_xalign(0.0);
        thinking_dropdown_header.append(&thinking_dropdown_label);

        let thinking_dropdown_arrow = gtk::Label::new(Some("stat_minus_1"));
        thinking_dropdown_arrow.set_css_classes(&["ai-chat-thinking-dropdown-arrow"]);
        thinking_dropdown_arrow.set_halign(gtk::Align::End);
        thinking_dropdown_arrow.set_hexpand(true);
        thinking_dropdown_arrow.set_xalign(1.0);
        thinking_dropdown_header.append(&thinking_dropdown_arrow);

        let thinking_dropdown_revealer = AdwRevealer::default();
        thinking_dropdown_revealer.set_css_classes(&["ai-chat-thinking-dropdown-revealer"]);
        thinking_dropdown_revealer.set_transition_direction(AdwRevealerDirection::Down);
        thinking_dropdown_revealer.set_show_easing(GEasing::EaseOutExpo);
        thinking_dropdown_revealer.set_hide_easing(GEasing::EaseOutExpo);
        thinking_dropdown_revealer.set_transition_duration(500);
        thinking_dropdown_revealer.set_reveal(false);
        root.append(&thinking_dropdown_revealer);

        let summary = gtk4cmark::MarkdownView::default();
        summary.set_css_classes(&["ai-chat-thinking-summary"]);
        summary.set_overflow(gtk::Overflow::Hidden);
        summary.set_vexpand(true);
        summary.set_hexpand(true);
        thinking_dropdown_revealer.set_child_from(Some(&summary));

        thinking_dropdown_button.connect_clicked(clone!(
            #[weak] root,
            move |_| {
                let currently_revealed = thinking_dropdown_revealer.reveal();
                thinking_dropdown_revealer.set_reveal(!currently_revealed);

                if currently_revealed {
                    root.remove_css_class("expanded");
                } else {
                    root.add_css_class("expanded");
                }
            }
        ));

        Self {
            root,
            summary_root: summary,
            summary: Rc::new(RefCell::new(String::new())),
        }
    }

    pub fn set_summary(&self, content: &str) {
        self.summary_root.set_markdown(content);
        content.clone_into(&mut self.summary.borrow_mut());
    }

    pub fn append_summary(&self, delta: &str) {
        let new_summary = format!("{}{}", self.summary.borrow(), delta);
        self.set_summary(&new_summary);
    }

    pub fn new_part(&self) {
        if !self.summary.borrow().is_empty() {
            self.append_summary("\n\n");
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChatContentElement {
    pub view: ChatMessageContent,
    pub item_id: Rc<Cell<Option<i64>>>,
}

impl ChatContentElement {
    pub fn new(content: &str, item_id: Option<i64>) -> Self {
        let item_id = Rc::new(Cell::new(item_id));

        let view = ChatMessageContent::new();
        view.set_content(content);

        view.connect_closure("save-edit", false, closure_local!(
            #[strong] item_id,
            move |view: ChatMessageContent| {
                if !ai::is_currently_in_cycle()
                    && let Some(item_id) = item_id.get()
                    && let Some(AiConversationItemPayload::Message { id, role, thought_signature, .. }) = ai::get_item_payload(item_id)
                {
                    let payload = AiConversationItemPayload::Message {
                        id,
                        content: view.content(),
                        role,
                        thought_signature,
                    };

                    tokio::spawn(ai::update_item(item_id, payload));
                }
            }
        ));

        Self {
            view,
            item_id,
        }
    }

    pub fn append_content(&self, delta: &str) {
        let new_content = format!("{}{}", self.view.content(), delta);
        self.view.set_content(new_content.as_str());
    }

    pub fn start_editing(&self) {
        if self.item_id.get().is_some() {
            self.view.set_editing(true);
        }
    }
}

pub fn tool_call(tool_name: &str, arguments: &str) -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root.set_css_classes(&["ai-chat-message-tool-call"]);

    let tool_call_button = gtk::Button::new();
    tool_call_button.set_css_classes(&["ai-chat-message-tool-call-button"]);
    tool_call_button.set_halign(gtk::Align::Fill);
    tool_call_button.set_hexpand(true);

    let tool_call_header = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    tool_call_header.set_hexpand(true);
    tool_call_button.set_child(Some(&tool_call_header));

    let tool_call_icon = gtk::Label::new(Some("build"));
    tool_call_icon.set_css_classes(&["ai-chat-message-tool-call-icon"]);
    tool_call_icon.set_halign(gtk::Align::Start);
    tool_call_icon.set_xalign(0.0);
    tool_call_header.append(&tool_call_icon);

    let tool_name_label = gtk::Label::new(Some(tool_name));
    tool_name_label.set_css_classes(&["ai-chat-message-tool-call-name"]);
    tool_name_label.set_halign(gtk::Align::Start);
    tool_name_label.set_xalign(0.0);
    tool_call_header.append(&tool_name_label);

    let tool_call_arrow = gtk::Label::new(Some("stat_minus_1"));
    tool_call_arrow.set_css_classes(&["ai-chat-message-tool-call-arrow"]);
    tool_call_arrow.set_halign(gtk::Align::End);
    tool_call_arrow.set_hexpand(true);
    tool_call_arrow.set_xalign(1.0);
    tool_call_header.append(&tool_call_arrow);

    let output_revealer = gtk::Revealer::new();
    output_revealer.set_reveal_child(false);

    let arguments_label = gtk::Label::new(Some(arguments));
    arguments_label.set_css_classes(&["ai-chat-message-tool-call-arguments"]);
    arguments_label.set_halign(gtk::Align::Start);
    arguments_label.set_xalign(0.0);
    output_revealer.set_child(Some(&arguments_label));

    tool_call_button.connect_clicked(clone!(
        #[weak] root,
        #[weak] output_revealer,
        move |_| {
            let revealed = output_revealer.reveals_child();
            output_revealer.set_reveal_child(!revealed);
            if revealed {
                root.remove_css_class("expanded");
            } else {
                root.add_css_class("expanded");
            }
        }
    ));

    root.append(&tool_call_button);
    root.append(&output_revealer);
    root
}

pub fn web_search_call() -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    root.set_css_classes(&["ai-chat-message-web-call"]);

    let web_call_icon = gtk::Label::new(Some("language"));
    web_call_icon.set_css_classes(&["ai-chat-message-web-call-icon"]);
    web_call_icon.set_halign(gtk::Align::Start);
    web_call_icon.set_xalign(0.0);

    let web_call_label = gtk::Label::new(Some("Searching the web..."));
    web_call_label.set_css_classes(&["ai-chat-message-web-call-label"]);
    web_call_label.set_halign(gtk::Align::Start);
    web_call_label.set_xalign(0.0);

    root.append(&web_call_icon);
    root.append(&web_call_label);
    root
}

pub fn image(uuid: &str) -> Option<gtk::Widget> {
    match gtk::gdk::Texture::from_filename(uuid_to_file_path(uuid)) {
        Ok(texture) => {
            let w_clamp = libadwaita::Clamp::new();
            w_clamp.set_maximum_size(300);
            w_clamp.set_unit(libadwaita::LengthUnit::Px);
            w_clamp.set_halign(gtk::Align::Start);
            w_clamp.set_valign(gtk::Align::Start);

            let h_clamp = libadwaita::Clamp::new();
            h_clamp.set_maximum_size(300);
            h_clamp.set_unit(libadwaita::LengthUnit::Px);
            h_clamp.set_orientation(gtk::Orientation::Vertical);
            w_clamp.set_child(Some(&h_clamp));

            let picture = gtk::Picture::new();
            picture.set_css_classes(&["ai-chat-message-image"]);
            picture.set_paintable(Some(&texture));
            picture.set_content_fit(gtk::ContentFit::ScaleDown);
            h_clamp.set_child(Some(&picture));

            Some(w_clamp.upcast())
        },

        Err(err) => {
            error!(uuid, ?err, "Failed to load image");
            None
        }
    }
}
