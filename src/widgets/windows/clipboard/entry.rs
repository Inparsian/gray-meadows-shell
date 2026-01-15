use gtk4::prelude::*;
use image::codecs::png::PngEncoder;
use image::imageops::FilterType;
use image::ImageEncoder as _;

use crate::color;
use crate::singletons::clipboard;
use crate::widgets::windows;

static IMAGE_SIZE: u32 = 192;
static IMAGE_SEMAPHORE: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(3);

pub fn clipboard_entry(id: usize, preview: &str) -> gtk4::Button {
    let button = gtk4::Button::new();
    button.set_css_classes(&["clipboard-entry"]);

    let children: Vec<gtk4::Widget> = if clipboard::is_an_image_clipboard_entry(preview) {
        let picture = gtk4::Picture::new();
        picture.set_halign(gtk4::Align::Start);
        picture.set_valign(gtk4::Align::Center);

        let (tx, rx) = async_channel::unbounded::<(u32, u32, Vec<u8>)>();
        tokio::spawn(async move {
            let _permit = IMAGE_SEMAPHORE.acquire().await.unwrap();
            if let Some(decoded) = clipboard::decode_clipboard_entry(&id.to_string()) {
                let image = image::load_from_memory(&decoded).ok();
                if let Some(img) = image {
                    let scaled_img = if img.width() > IMAGE_SIZE || img.height() > IMAGE_SIZE {
                        let aspect = img.width() as f32 / img.height() as f32;
                        let new_width = if aspect >= 1.0 {
                            IMAGE_SIZE
                        } else {
                            (IMAGE_SIZE as f32 * aspect) as u32
                        };
                        let new_height = if aspect >= 1.0 {
                            (IMAGE_SIZE as f32 / aspect) as u32
                        } else {
                            IMAGE_SIZE
                        };
                        // TODO: add a way to cache image clipboard entry thumbnails
                        img.resize(new_width, new_height, FilterType::Nearest)
                    } else {
                        img
                    };

                    let mut buf = Vec::new();
                    if PngEncoder::new(&mut buf).write_image(
                        scaled_img.as_bytes(),
                        scaled_img.width(),
                        scaled_img.height(),
                        scaled_img.color().into()
                    ).is_ok() {
                        let _ = tx.send((scaled_img.width(), scaled_img.height(), buf)).await;
                    }
                }
            }
        });

        gtk4::glib::spawn_future_local({
            let picture = picture.clone();
            async move {
                if let Ok((width, height, decoded)) = rx.recv().await {
                    let loader = gtk4::gdk_pixbuf::PixbufLoader::new();
                    if loader.write(&decoded).is_ok() {
                        let _ = loader.close();
                        if let Some(pixbuf) = loader.pixbuf() {
                            picture.set_pixbuf(Some(&pixbuf));
                            picture.set_width_request(width as i32);
                            picture.set_height_request(height as i32);
                        }
                    }
                }
            }
        });

        vec![picture.upcast()]
    } else if let Some(hex) = color::parse_color_into_hex(preview) {
        let color_style_provider = gtk4::CssProvider::new();
        let color_style = format!(".color-preview-box {{ background-color: {}; }}", hex);
        color_style_provider.load_from_data(&color_style);

        let color_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        color_box.set_css_classes(&["color-preview-box"]);
        color_box.set_valign(gtk4::Align::Center);
        color_box.style_context().add_provider(&color_style_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

        let label = gtk4::Label::new(Some(preview));
        label.set_hexpand(true);
        label.set_xalign(0.0);
        label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        vec![color_box.upcast(), label.upcast()]
    } else {
        // glib hates nul bytes where gstrings do not actually end :)
        let preview_cleaned = preview.chars().filter(|c| c != &'\0').collect::<String>();
        let label = gtk4::Label::new(Some(&preview_cleaned));
        label.set_hexpand(true);
        label.set_xalign(0.0);
        label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        vec![label.upcast()]
    };

    button.connect_clicked(move |_| {
        clipboard::copy_entry(id);
        windows::hide("clipboard");
    });

    // stupid layout trick that helps the button not vertically stretch more than needed
    let bx = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    for child in &children {
        bx.append(child);
    }
    let bxend = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    bxend.set_halign(gtk4::Align::End);
    bx.append(&bxend);
    button.set_child(Some(&bx));

    button
}