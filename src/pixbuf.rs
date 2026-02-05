use gdk::gdk_pixbuf::Pixbuf;
use gio::prelude::FileExt as _;
use gtk::TextDirection;
use gtk::gdk_pixbuf::InterpType;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;

use crate::APP_LOCAL;

const PIXBUF_WIDTH: i32 = 24;
const PIXBUF_HEIGHT: i32 = 24;

thread_local! {
    pub static PIXBUF_CACHE: RefCell<HashMap<String, Pixbuf>> = RefCell::new(HashMap::new());
}

pub fn get_pixbuf(icon_name: &str) -> Option<Pixbuf> {
    // Ensure this is being run on the main context.
    if !glib::MainContext::default().is_owner() {
        return None;
    }

    PIXBUF_CACHE.with(|pixbufs| {
        let mut pixbufs = pixbufs.borrow_mut();
        if let Some(pixbuf) = pixbufs.get(icon_name) {
            return Some(pixbuf.clone());
        }

        // Is this a path to an image file (e.g., /path/to/image.png)?
        if Path::new(icon_name).exists() 
            && let Ok(pixbuf) = Pixbuf::from_file(icon_name)
            && let Some(pixbuf) = pixbuf.scale_simple(
                PIXBUF_WIDTH,
                PIXBUF_HEIGHT,
                InterpType::Bilinear
            )
        {
            pixbufs.insert(icon_name.to_owned(), pixbuf.clone());
            return Some(pixbuf);
        }

        // Otherwise, try to load it as an icon from the icon theme
        let icon_paintable = APP_LOCAL.with(|app| app.icon_theme.lookup_icon(
            icon_name,
            &[],
            0, 
            1,
            TextDirection::Ltr,
            gtk::IconLookupFlags::empty()
        ));

        if let Some(path) = icon_paintable.file().and_then(|f| f.path())
            && let Ok(pixbuf) = Pixbuf::from_file(path)
            && let Some(pixbuf) = pixbuf.scale_simple(
                PIXBUF_WIDTH, 
                PIXBUF_HEIGHT,
                InterpType::Bilinear
            )
        {
            pixbufs.insert(icon_name.to_owned(), pixbuf.clone());

            Some(pixbuf)
        } else {
            None
        }
    })
}

pub fn get_pixbuf_or_fallback(icon_name: &str, fallback_name: &str) -> Option<Pixbuf> {
    get_pixbuf(icon_name).map_or_else(|| get_pixbuf(fallback_name), Some)
}