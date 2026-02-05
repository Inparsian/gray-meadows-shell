use super::wrapper::sn_item::RawPixmap;

// Rationale: Some icons have the possibility of being absurdly large (e.g. 1024x1024). This may not seem like an
// issue at first, however, compared to a measly 28x28 icon, which has a total of 3136 (28^2*4) ARGB values, a
// 1024x1024 icon has a total of 4,194,304 (1024^2*4) ARGB values.
//
// That is a LOT of data to store in memory AND convert to a Pixbuf. Not only does this lead to huge performance
// issues, but the detail in icons that large are imperceptible when they're resized to a smaller size, so this also
// leads to unnecessary memory usage.
const C_WIDTH: u32 = 28;
const C_HEIGHT: u32 = 28;

pub fn compress_icon_pixmap(pixmap: Option<&Vec<RawPixmap>>) -> Option<Vec<RawPixmap>> {
    let argb32_icon = pixmap?;
    
    // Pick the icon that is closest to C_WIDTHxC_HEIGHT.
    let closest_icon = argb32_icon.iter()
        .min_by_key(|pixmap| {
            let width = pixmap.0;
            let height = pixmap.1;

            (width - C_WIDTH as i32).abs() + (height - C_HEIGHT as i32).abs()
        });

    closest_icon.map(|icon| {
        // Perform pixel compression if icon.width and icon.height are larger than C_WIDTH and C_HEIGHT
        let should_compress = icon.0 > C_WIDTH as i32 || icon.1 > C_HEIGHT as i32;
        let compressed_pixels = if should_compress {
            let mut vec = Vec::new();

            for y in 0..C_HEIGHT {
                for x in 0..C_WIDTH {
                    let c_icon_x = (x as f32 / C_WIDTH as f32 * icon.0 as f32) as u32;
                    let c_icon_y = (y as f32 / C_HEIGHT as f32 * icon.1 as f32) as u32;
                    let c_icon_index = (c_icon_y * icon.0 as u32 + c_icon_x) as usize * 4;

                    if c_icon_index < icon.2.len() {
                        // push the next 4 items (a, r, g, b)
                        for c in 0..4 {
                            let pixel_index = c_icon_index + c;
                            if pixel_index < icon.2.len() {
                                vec.push(icon.2[pixel_index]);
                            }
                        }
                    }
                }
            }

            vec
        } else {
            icon.2.clone() // leave as is
        };

        vec![(
            if should_compress {
                C_WIDTH as i32
            } else {
                icon.0
            },

            if should_compress {
                C_HEIGHT as i32
            } else {
                icon.1
            },

            compressed_pixels
        )]
    })
}

pub fn make_icon_pixbuf(pixmap: Option<&Vec<RawPixmap>>) -> Option<gtk::gdk_pixbuf::Pixbuf> {
    let argb32_icon = pixmap?;
    let closest_icon = argb32_icon.iter()
        .min_by_key(|pixmap| {
            let width = pixmap.0;
            let height = pixmap.1;

            (width - C_WIDTH as i32).abs() + (height - C_HEIGHT as i32).abs()
        });

    closest_icon.map(|icon| {
        let pixbuf = gtk::gdk_pixbuf::Pixbuf::from_mut_slice(
            icon.2.clone(),
            gtk::gdk_pixbuf::Colorspace::Rgb,
            true,
            8,
            icon.0,
            icon.1,
            icon.0 * 4
        );

        // aesthetic thing
        pixbuf.saturate_and_pixelate(
            &pixbuf,
            0.0,
            false
        );

        pixbuf
    })
}