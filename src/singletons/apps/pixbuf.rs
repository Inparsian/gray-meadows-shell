use gdk4::gdk_pixbuf::Pixbuf;
use gdk4::gio::prelude::FileExt;
use gtk4::TextDirection;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::APP;

thread_local! {
    pub static PIXBUF_CACHE: RefCell<HashMap<String, Pixbuf>> = RefCell::new(HashMap::new());
}

pub fn get_pixbuf(icon_name: &str) -> Option<Pixbuf> {
    // Ensure this is being run on the main context.
    if !gtk4::glib::MainContext::default().is_owner() {
        return None;
    }

    PIXBUF_CACHE.with(|pixbufs| {
        let mut pixbufs = pixbufs.borrow_mut();
        if let Some(pixbuf) = pixbufs.get(icon_name) {
            return Some(pixbuf.clone());
        }

        // Is this a path to an image file (e.g., /path/to/image.png)?
        if std::path::Path::new(icon_name).exists() {
            if let Ok(pixbuf) = Pixbuf::from_file(icon_name) {
                let pixbuf = pixbuf.scale_simple(24, 24, gtk4::gdk_pixbuf::InterpType::Bilinear);
                
                if let Some(pixbuf) = pixbuf {
                    pixbufs.insert(icon_name.to_owned(), pixbuf.clone());
                    return Some(pixbuf);
                }
            }
        }

        // Otherwise, try to load it as an icon from the icon theme
        let icon_theme = APP.with(|app| app.borrow().icon_theme.clone());
        let icon_paintable = icon_theme.lookup_icon(
            icon_name,
            &[],
            0, 1,
            TextDirection::Ltr,
            gtk4::IconLookupFlags::empty()
        );

        if let Some(path) = icon_paintable.file().and_then(|f| f.path()) {
            if let Ok(pixbuf) = Pixbuf::from_file(path) {
                let pixbuf = pixbuf.scale_simple(24, 24, gtk4::gdk_pixbuf::InterpType::Bilinear);
                
                if let Some(pixbuf) = pixbuf {
                    pixbufs.insert(icon_name.to_owned(), pixbuf.clone());
                    return Some(pixbuf);
                }
            }
        }

        None
    })
}

pub fn get_pixbuf_or_fallback(icon_name: &str, fallback_name: &str) -> Option<Pixbuf> {
    get_pixbuf(icon_name).map_or_else(|| get_pixbuf(fallback_name), Some)
}