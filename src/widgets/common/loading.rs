use std::cell::RefCell;
use std::rc::Rc;
use gtk::prelude::*;

use crate::scss;

pub fn new() -> gtk::DrawingArea {
    let frametime = Rc::new(RefCell::new(0.0));
    let drawing_area = gtk::DrawingArea::new();
    drawing_area.set_size_request(40, 20);
    drawing_area.set_css_classes(&["loading-indicator"]);

    // draw cool animated loading boxes that move up and down
    drawing_area.set_draw_func(clone!(
        #[strong] frametime,
        move |_, cr, width, height| {
            let time: f64 = *frametime.borrow();
            let box_width = width as f64 / 6.0;
            let box_height = height as f64 / 4.0;

            for i in 0..5 {
                let phase = time.mul_add(5.0, -(i as f64)).rem_euclid(5.0);
                let y_offset = if phase < 1.0 {
                    phase * box_height
                } else if phase < 2.0 {
                    (2.0 - phase) * box_height
                } else {
                    0.0
                };

                let (r, g, b) = scss::get_color("foreground-color-primary").map_or(
                    (226.0, 226.0, 226.0), 
                    |box_color| (box_color.red as f64, box_color.green as f64, box_color.blue as f64)
                );

                cr.set_source_rgb(r / 255.0, g / 255.0, b / 255.0);
                cr.rectangle(
                    (i as f64 * (width as f64 / 5.0)).mul_add(1.0, width as f64 * 0.05),
                    height as f64 - box_height - y_offset,
                    box_width * 0.6,
                    box_height * 0.8
                );

                let _ = cr.fill();
            }
        }
    ));

    drawing_area.add_tick_callback(move |drawing_area, clock| {
        if drawing_area.is_visible() {
            *frametime.borrow_mut() = clock.frame_time() as f64 / 1_000_000.0;
            drawing_area.queue_draw();
        }
        
        glib::ControlFlow::Continue
    });
    
    drawing_area
}