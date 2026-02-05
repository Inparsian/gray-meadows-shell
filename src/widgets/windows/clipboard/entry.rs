mod imp {
    use std::cell::Cell;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    
    use crate::services::clipboard;
    use crate::widgets::windows;
    
    #[derive(glib::Properties, Default)]
    #[properties(wrapper_type = super::ClipboardEntry)]
    pub struct ClipboardEntry {
        #[property(get, set)]
        pub id: Cell<i32>,
    }
    
    #[glib::object_subclass]
    impl ObjectSubclass for ClipboardEntry {
        const NAME: &'static str = "ClipboardEntry";
        type Type = super::ClipboardEntry;
        type ParentType = gtk::Button;
    }
    
    #[glib::derived_properties]
    impl ObjectImpl for ClipboardEntry {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }
    
    impl WidgetImpl for ClipboardEntry {}
    
    impl ButtonImpl for ClipboardEntry {
        fn clicked(&self) {
            let id = self.id.get();
            if id > -1 {
                clipboard::copy_entry(id);
                windows::hide("clipboard");
            }
        }
    }
}

use gtk::prelude::*;

use crate::color;
use crate::services::clipboard;
use crate::widgets::common::loading;

glib::wrapper! {
    pub struct ClipboardEntry(ObjectSubclass<imp::ClipboardEntry>)
        @extends gtk::Button, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for ClipboardEntry {
    fn default() -> Self {
        glib::Object::builder()
            .property("id", -1)
            .build()
    }
}

impl ClipboardEntry {
    pub fn refresh(&self) {
        let id = self.id();
        if id < 0 {
            return;
        }
        
        let Some(preview) = clipboard::get_preview(id) else {
            return;
        };
        
        let bx = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        if clipboard::images::is_an_image_clipboard_entry(&preview) {
            let preview_data = clipboard::images::ImagePreviewData::from_clipboard_preview(&preview).unwrap();
            let (width, height) = clipboard::images::get_downscale_image_resolution(preview_data.width, preview_data.height);
            let loading_bx = gtk::Box::new(gtk::Orientation::Vertical, 0);
            loading_bx.set_css_classes(&["image-preview-loading-box"]);
            loading_bx.set_halign(gtk::Align::Start);
            loading_bx.set_valign(gtk::Align::Center);
            loading_bx.set_size_request(width as i32, height as i32);
            let loading_widget = loading::new();
            loading_widget.set_halign(gtk::Align::Center);
            loading_widget.set_valign(gtk::Align::Center);
            loading_widget.set_vexpand(true);
            loading_bx.append(&loading_widget);
            
            let picture = gtk::Picture::new();
            picture.set_visible(false);
            picture.set_halign(gtk::Align::Start);
            picture.set_valign(gtk::Align::Center);
    
            let (tx, rx) = async_channel::unbounded::<(u32, u32, Vec<u8>)>();
            tokio::spawn(async move {
                if let Some(image) = clipboard::images::get_image_entry(id).await {
                    let _ = tx.send(image).await;
                }
            });
            
            bx.append(&loading_bx);
            bx.append(&picture);
    
            glib::spawn_future_local(clone!(
                #[weak] picture,
                #[weak] loading_bx,
                async move {
                    if let Ok((width, height, decoded)) = rx.recv().await {
                        let loader = gtk::gdk_pixbuf::PixbufLoader::new();
                        if loader.write(&decoded).is_ok() {
                            let _ = loader.close();
                            if let Some(pixbuf) = loader.pixbuf() {
                                picture.set_pixbuf(Some(&pixbuf));
                                picture.set_width_request(width as i32);
                                picture.set_height_request(height as i32);
                                picture.set_visible(true);
                                loading_bx.set_visible(false);
                            }
                        }
                    }
                }
            ));
        } else if let Some(hex) = color::parse_color_into_hex(&preview) {
            let color_style_provider = gtk::CssProvider::new();
            let color_style = format!(".color-preview-box {{ background-color: {}; }}", hex);
            color_style_provider.load_from_data(&color_style);
    
            let color_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
            color_box.set_css_classes(&["color-preview-box"]);
            color_box.set_valign(gtk::Align::Center);
            color_box.style_context().add_provider(&color_style_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    
            let label = gtk::Label::new(Some(&preview));
            label.set_hexpand(true);
            label.set_xalign(0.0);
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            bx.append(&color_box);
            bx.append(&label);
        } else {
            // glib hates nul bytes where gstrings do not actually end :)
            let preview_cleaned = preview.chars().filter(|c| c != &'\0').collect::<String>();
            let label = gtk::Label::new(Some(&preview_cleaned));
            label.set_hexpand(true);
            label.set_xalign(0.0);
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            bx.append(&label);
        }
    
        // stupid layout trick that helps the button not vertically stretch more than needed
        let bxend = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        bxend.set_halign(gtk::Align::End);
        bx.append(&bxend);
        self.set_child(Some(&bx));
    }
}