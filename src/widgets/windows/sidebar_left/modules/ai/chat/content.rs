mod imp {
    use std::rc::Rc;
    use std::cell::{Cell, RefCell};
    use std::sync::OnceLock;
    use glib::subclass::Signal;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use glib::Properties;
    
    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::ChatMessageContent)]
    pub struct ChatMessageContent {
        #[property(get, set = Self::set_editing)]
        pub editing: Cell<bool>,
        #[property(get, set = Self::set_content)]
        pub content: RefCell<String>,
        
        // inner widgets
        pub(super) markdown_view: Rc<RefCell<Option<gtk4cmark::MarkdownView>>>,
        pub(super) edit_view: Rc<RefCell<Option<gtk::TextView>>>,
        pub(super) edit_box: Rc<RefCell<Option<gtk::Box>>>,
    }
    
    #[glib::object_subclass]
    impl ObjectSubclass for ChatMessageContent {
        const NAME: &'static str = "ChatMessageContent";
        type Type = super::ChatMessageContent;
        type ParentType = gtk::Box;
    }
    
    #[glib::derived_properties]
    impl ObjectImpl for ChatMessageContent {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder("save-edit").build(),
                ]
            })
        }
        
        fn constructed(&self) {
            self.parent_constructed();
            
            let obj = self.obj();
            obj.set_hexpand(true);
            obj.set_vexpand(true);
            
            let markdown = gtk4cmark::MarkdownView::default();
            markdown.set_css_classes(&["ai-chat-message-content"]);
            markdown.set_overflow(gtk::Overflow::Hidden);
            markdown.set_vexpand(true);
            markdown.set_hexpand(true);
            
            let edit = gtk::TextView::new();
            edit.set_css_classes(&["ai-chat-message-content-edit"]);
            edit.set_overflow(gtk::Overflow::Hidden);
            edit.set_wrap_mode(gtk::WrapMode::WordChar);
            edit.set_vexpand(true);
            edit.set_hexpand(true);
            
            let edit_scrolled = gtk::ScrolledWindow::new();
            edit_scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
            edit_scrolled.set_max_content_height(300);
            edit_scrolled.set_propagate_natural_height(true);
            edit_scrolled.set_child(Some(&edit));
            
            let edit_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
            edit_box.append(&edit_scrolled);
            edit_box.set_hexpand(true);
            edit_box.set_vexpand(true);
            edit_box.set_visible(false);
            
            let edit_buttons_group = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            edit_buttons_group.set_halign(gtk::Align::End);
            edit_buttons_group.set_hexpand(true);
            edit_box.append(&edit_buttons_group);
            
            let discard_button = gtk::Button::new();
            discard_button.set_label("Discard");
            discard_button.set_css_classes(&["ai-chat-message-content-edit-button"]);
            discard_button.set_vexpand(false);
            discard_button.set_hexpand(false);
            discard_button.connect_clicked(glib::clone!(
                #[weak(rename_to = imp)] self,
                move |_| {
                    imp.discard_edit();
                }
            ));
            edit_buttons_group.append(&discard_button);
            
            let save_button = gtk::Button::new();
            save_button.set_label("Save");
            save_button.set_css_classes(&["ai-chat-message-content-edit-button", "save"]);
            save_button.set_vexpand(false);
            save_button.set_hexpand(false);
            save_button.connect_clicked(glib::clone!(
                #[weak(rename_to = imp)] self,
                move |_| {
                    imp.save_edit();
                }
            ));
            edit_buttons_group.append(&save_button);
            
            obj.append(&markdown);
            obj.append(&edit_box);
            
            self.markdown_view.replace(Some(markdown));
            self.edit_view.replace(Some(edit));
            self.edit_box.replace(Some(edit_box));
        }
    }
    
    impl WidgetImpl for ChatMessageContent {}
    
    impl BoxImpl for ChatMessageContent {}
    
    impl ChatMessageContent {
        fn save_edit(&self) {
            if !self.editing.get() {
                return;
            }
            
            if let Some(edit_view) = self.edit_view.borrow().as_ref() {
                let buffer = edit_view.buffer();
                let start = buffer.start_iter();
                let end = buffer.end_iter();
                let content = buffer.text(&start, &end, false).to_string();
                
                self.content.replace(content.clone());
                
                if let Some(markdown_view) = self.markdown_view.borrow().as_ref() {
                    markdown_view.set_markdown(content);
                }
            }
            
            self.obj().emit_by_name::<()>("save-edit", &[]);
            self.set_editing(false);
        }
        
        fn discard_edit(&self) {
            if !self.editing.get() {
                return;
            }
            
            if let Some(edit_view) = self.edit_view.borrow().as_ref() {
                edit_view.buffer().set_text(&self.content.borrow());
            }
            
            self.set_editing(false);
        }
        
        fn set_content(&self, content: &str) {
            if *self.content.borrow() == content {
                return;
            }
            
            self.content.replace(content.to_owned());
            
            if let Some(markdown_view) = self.markdown_view.borrow().as_ref() {
                markdown_view.set_markdown(content);
            }
        }
        
        fn set_editing(&self, editing: bool) {
            if self.editing.get() == editing {
                return;
            }
            
            self.editing.set(editing);
            
            if let Some(markdown_view) = self.markdown_view.borrow().as_ref() {
                markdown_view.set_visible(!editing);
            }
            
            if let Some(edit_box) = self.edit_box.borrow().as_ref() {
                edit_box.set_visible(editing);
                
                if let Some(edit_view) = self.edit_view.borrow().as_ref() && editing {
                    edit_view.buffer().set_text(&self.content.borrow());
                }
            }
        }
    }
}

use gtk::prelude::*;
use glib::Object;
use glib::subclass::types::ObjectSubclassIsExt as _;

glib::wrapper! {
    pub struct ChatMessageContent(ObjectSubclass<imp::ChatMessageContent>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ChatMessageContent {
    pub fn new() -> Self {
        Object::builder().build()
    }
    
    pub fn get_edit_content(&self) -> Option<String> {
        let edit_view = self.edit_view()?;
        let buffer = edit_view.buffer();
        let start = buffer.start_iter();
        let end = buffer.end_iter();
        Some(buffer.slice(&start, &end, false).to_string())
    }
    
    pub fn markdown_view(&self) -> Option<gtk4cmark::MarkdownView> {
        self.imp().markdown_view.borrow().clone()
    }
    
    pub fn edit_view(&self) -> Option<gtk::TextView> {
        self.imp().edit_view.borrow().clone()
    }
}