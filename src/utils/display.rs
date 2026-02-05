use gdk::prelude::{Cast as _, DisplayExt as _};

pub fn get_all_monitors(display: &gdk::Display) -> Vec<gdk::Monitor> {
    let monitors = gdk::Display::monitors(display);

    monitors
        .into_iter()
        .filter_map(|monitor| {
            match monitor {
                Ok(m) => m.downcast::<gdk::Monitor>().ok(),
                Err(e) => {
                    warn!(?e, "Error iterating monitors");
                    None
                }
            }
        })
        .collect()
}