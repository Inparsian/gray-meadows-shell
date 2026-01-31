mod imp {
    use std::cell::Cell;
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;
    
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
        type ParentType = gtk4::Button;
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

use gtk4::prelude::*;

use crate::color;
use crate::services::clipboard;
use crate::widgets::common::loading;

glib::wrapper! {
    pub struct ClipboardEntry(ObjectSubclass<imp::ClipboardEntry>)
        @extends gtk4::Button, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Actionable, gtk4::Buildable, gtk4::ConstraintTarget;
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
        
        let bx = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        if clipboard::images::is_an_image_clipboard_entry(&preview) {
            let preview_data = clipboard::images::ImagePreviewData::from_clipboard_preview(&preview).unwrap();
            let (width, height) = clipboard::images::get_downscale_image_resolution(preview_data.width, preview_data.height);
            let loading_bx = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
            loading_bx.set_css_classes(&["image-preview-loading-box"]);
            loading_bx.set_halign(gtk4::Align::Start);
            loading_bx.set_valign(gtk4::Align::Center);
            loading_bx.set_size_request(width as i32, height as i32);
            let loading_widget = loading::new();
            loading_widget.set_halign(gtk4::Align::Center);
            loading_widget.set_valign(gtk4::Align::Center);
            loading_widget.set_vexpand(true);
            loading_bx.append(&loading_widget);
            
            let picture = gtk4::Picture::new();
            picture.set_visible(false);
            picture.set_halign(gtk4::Align::Start);
            picture.set_valign(gtk4::Align::Center);
    
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
                        let loader = gtk4::gdk_pixbuf::PixbufLoader::new();
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
            let color_style_provider = gtk4::CssProvider::new();
            let color_style = format!(".color-preview-box {{ background-color: {}; }}", hex);
            color_style_provider.load_from_data(&color_style);
    
            let color_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
            color_box.set_css_classes(&["color-preview-box"]);
            color_box.set_valign(gtk4::Align::Center);
            color_box.style_context().add_provider(&color_style_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
    
            let label = gtk4::Label::new(Some(&preview));
            label.set_hexpand(true);
            label.set_xalign(0.0);
            label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            bx.append(&color_box);
            bx.append(&label);
        } else {
            // glib hates nul bytes where gstrings do not actually end :)
            let preview_cleaned = preview.chars().filter(|c| c != &'\0').collect::<String>();
            let label = gtk4::Label::new(Some(&preview_cleaned));
            label.set_hexpand(true);
            label.set_xalign(0.0);
            label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            bx.append(&label);
        }
    
        // stupid layout trick that helps the button not vertically stretch more than needed
        let bxend = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        bxend.set_halign(gtk4::Align::End);
        bx.append(&bxend);
        self.set_child(Some(&bx));
    }
}