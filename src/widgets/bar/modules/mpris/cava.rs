use gtk::prelude::*;

#[cfg(feature = "cava")]
mod enabled {
    use std::sync::Once;

    use super::*;
    use crate::ffi::astalcava::ffi;

    static INITIALIZE_CAVA: Once = Once::new();

    pub fn new(alpha: f64) -> gtk::DrawingArea {
        INITIALIZE_CAVA.call_once(ffi::init_cava);

        let drawing_area = gtk::DrawingArea::new();
        drawing_area.set_css_classes(&["bar-mpris-cava"]);
        drawing_area.set_hexpand(true);
        drawing_area.set_vexpand(true);
        drawing_area.set_can_target(false);
        let alpha = alpha.clamp(0.0, 1.0);
        drawing_area.set_draw_func(move |_, cr, width, height| {
            let values = ffi::get_values();
            if values.is_empty() || width <= 0 || height <= 0 {
                return;
            }

            let width = width as f64;
            let height = height as f64;
            let step = width / (values.len().saturating_sub(1).max(1) as f64);
            let points: Vec<_> = values
                .iter()
                .enumerate()
                .map(|(index, value)| (step * index as f64, (1.0 - value / 1.15) * height))
                .collect();

            cr.new_path();
            cr.move_to(0.0, height);
            cr.line_to(points[0].0, points[0].1);

            for pair in points.windows(2) {
                let current = pair[0];
                let next = pair[1];
                let control_x = f64::midpoint(current.0, next.0);
                let control_y = f64::midpoint(current.1, next.1);
                cr.curve_to(current.0, current.1, control_x, control_y, next.0, next.1);
            }

            cr.line_to(width, height);
            cr.close_path();
            cr.set_source_rgba(1.0, 1.0, 1.0, alpha);
            let _ = cr.fill();
        });

        drawing_area.add_tick_callback(|drawing_area, _| {
            if drawing_area.is_visible() {
                drawing_area.queue_draw();
            }

            glib::ControlFlow::Continue
        });

        drawing_area
    }
}

#[cfg(feature = "cava")]
pub use enabled::new;

#[cfg(not(feature = "cava"))]
pub fn new(_alpha: f64) -> gtk::DrawingArea {
    let drawing_area = gtk::DrawingArea::new();
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);
    drawing_area.set_can_target(false);
    drawing_area
}
