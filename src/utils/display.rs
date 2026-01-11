use gdk4::prelude::{Cast as _, DisplayExt as _};

pub fn get_all_monitors(display: &gdk4::Display) -> Vec<gdk4::Monitor> {
    let monitors = gdk4::Display::monitors(display);

    monitors
        .into_iter()
        .filter_map(|monitor| {
            match monitor {
                Ok(m) => m.downcast::<gdk4::Monitor>().ok(),
                Err(e) => {
                    eprintln!("Error iterating monitors: {:?}", e);
                    None
                }
            }
        })
        .collect()
}