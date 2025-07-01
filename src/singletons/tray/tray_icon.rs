use gdk4::glib::Bytes;
use system_tray::{item::{IconPixmap, StatusNotifierItem}};

// Rationale: Some icons have the possibility of being absurdly large (e.g. 1024x1024). This may not seem like an
// issue at first, however, compared to a measly 32x32 icon, which has a total of 4096 (32^2*4) ARGB values, a
// 1024x1024 icon has a total of 4,194,304 (1024^2*4) ARGB values.
// That is a LOT of data to store in memory AND convert to a Pixbuf. Not only does this lead to huge performance
// issues, but the detail in icons that large are imperceptible when they're resized to a smaller size, so this also
// leads to unnecessary memory usage. This is fairly trivial to fix however.
const C_WIDTH: u32 = 32;
const C_HEIGHT: u32 = 32;

pub fn compress_icon_pixmap(pixmap: &Option<Vec<IconPixmap>>) -> Option<Vec<IconPixmap>> {
    if let Some(argb32_icon) = pixmap {
        let closest_icon = argb32_icon.iter()
            .min_by_key(|pixmap| {
                let width = pixmap.width;
                let height = pixmap.height;

                (width - C_WIDTH as i32).abs() + (height - C_HEIGHT as i32).abs()
            });

        if let Some(icon) = closest_icon {
            // Perform pixel compression if icon.width and icon.height are larger than C_WIDTH and C_HEIGHT
            let should_compress = icon.width > C_WIDTH as i32 || icon.height > C_HEIGHT as i32;
            let compressed_pixels = if should_compress {
                let mut vec = Vec::new();

                for y in 0..C_HEIGHT {
                    for x in 0..C_WIDTH {
                        let c_icon_x = (x as f32 / C_WIDTH as f32 * icon.width as f32) as u32;
                        let c_icon_y = (y as f32 / C_HEIGHT as f32 * icon.height as f32) as u32;
                        let c_icon_index = (c_icon_y * icon.width as u32 + c_icon_x) as usize * 4;

                        if c_icon_index < icon.pixels.len() {
                            // push the next 4 items (a, r, g, b)
                            for c in 0..4 {
                                let pixel_index = c_icon_index + c;
                                if pixel_index < icon.pixels.len() {
                                    vec.push(icon.pixels[pixel_index]);
                                }
                            }
                        }
                    }
                }

                vec
            } else {
                icon.pixels.clone() // leave as is
            };

            Some(vec![IconPixmap {
                width: if should_compress {
                    C_WIDTH as i32
                } else {
                    icon.width
                },

                height: if should_compress {
                    C_HEIGHT as i32
                } else {
                    icon.height
                },

                pixels: compressed_pixels
            }])
        } else {
            None
        }
    } else {
        None
    }
}

pub fn compress_icon(item: &mut StatusNotifierItem) {
    if let Some(compressed_pixmap) = compress_icon_pixmap(&item.icon_pixmap) {
        item.icon_pixmap = Some(compressed_pixmap);
    }

    if let Some(compressed_pixmap) = compress_icon_pixmap(&item.overlay_icon_pixmap) {
        item.overlay_icon_pixmap = Some(compressed_pixmap);
    }

    if let Some(compressed_pixmap) = compress_icon_pixmap(&item.attention_icon_pixmap) {
        item.attention_icon_pixmap = Some(compressed_pixmap);
    }
}

pub fn make_icon_pixbuf(item: StatusNotifierItem) -> gtk4::gdk_pixbuf::Pixbuf {
    let argb32_icon = item.icon_pixmap.clone().unwrap_or_default();

    // Pick the icon that is closest to C_WIDTHxC_HEIGHT.
    let closest_icon = argb32_icon.iter()
        .min_by_key(|pixmap| {
            let width = pixmap.width;
            let height = pixmap.height;

            (width - C_WIDTH as i32).abs() + (height - C_HEIGHT as i32).abs()
        });

    if let Some(icon) = closest_icon {
        let pixbuf = gtk4::gdk_pixbuf::Pixbuf::from_mut_slice(
            icon.pixels.clone(),
            gtk4::gdk_pixbuf::Colorspace::Rgb,
            true,
            8,
            icon.width,
            icon.height,
            icon.width * 4
        );

        // aesthetic thing
        pixbuf.saturate_and_pixelate(
            &pixbuf,
            0.0,
            false
        );

        pixbuf
    } else {
        println!("No suitable icon found, returning blank pixbuf.");

        gtk4::gdk_pixbuf::Pixbuf::from_bytes(
            &Bytes::from(&[]),
            gtk4::gdk_pixbuf::Colorspace::Rgb,
            true,
            8,
            0,
            0,
            0
        )
    }
}