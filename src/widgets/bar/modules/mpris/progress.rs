use std::{cell::RefCell, f64, rc::Rc};
use gtk::prelude::*;

use crate::scss;
use crate::utils::gesture;
use crate::services::mpris;
use super::extended::SEEK_STEP_MICROSECONDS;

static WAVE_AMPLITUDE_FACTOR: f64 = 10.0;
static WAVE_SPEED: f64 = 3.0;
static WAVE_LINE_WIDTH: f64 = 3.0;

#[derive(Debug, Clone, glib::Downgrade)]
pub struct ProgressBar {
    pub drawing_area: gtk::DrawingArea,
    pub position: Rc<RefCell<f64>>,
    pub duration: Rc<RefCell<f64>>,
}

impl ProgressBar {
    pub fn new() -> Self {
        let frametime = Rc::new(RefCell::new(0.0));
        let seek_position: Rc<RefCell<Option<f64>>> = Rc::new(RefCell::new(None));
        let position = Rc::new(RefCell::new(0.0));
        let duration = Rc::new(RefCell::new(0.0));
        let drawing_area = gtk::DrawingArea::new();
        drawing_area.set_css_classes(&["bar-mpris-extended-progress-bar"]);
        drawing_area.set_content_width(200);
        drawing_area.set_content_height(24);

        // draw cool animated wavey progress bar
        drawing_area.set_draw_func(clone!(
            #[strong] position,
            #[strong] seek_position,
            #[strong] duration,
            #[strong] frametime,
            move |_, cr, width, height| {
                let pos: f64 = seek_position.borrow().map_or_else(|| *position.borrow(), |seek_pos| seek_pos);
                let dur: f64 = *duration.borrow();
                let time: f64 = *frametime.borrow();
                if dur <= 0.0 {
                    return;
                }

                let progress = (pos / dur).clamp(0.0, 1.0);
                let v_center = height as f64 / 2.0;
                let wave_amplitude = height as f64 / WAVE_AMPLITUDE_FACTOR;
                let wave_offset = |x: f64| time.mul_add(WAVE_SPEED, x / 20.0).sin() * wave_amplitude;

                let stroke = |alpha: f64, to: i32| {
                    let (r, g, b) = scss::get_color("foreground-color-primary").map_or(
                        (226.0, 226.0, 226.0), 
                        |wave_color| (wave_color.red as f64, wave_color.green as f64, wave_color.blue as f64)
                    );

                    cr.set_source_rgba(r / 255.0, g / 255.0, b / 255.0, alpha);
                    cr.set_line_width(WAVE_LINE_WIDTH);
                    cr.move_to(0.0, v_center);
                    for x in 0..=to {
                        cr.line_to(x as f64, v_center + wave_offset(x as f64));
                    }
                    let _ = cr.stroke();
                };

                let progress_width = (width as f64 * progress) as i32;
                stroke(0.3, width); // background wave
                stroke(1.0, progress_width); // foreground wave
            }
        ));
        
        drawing_area.add_tick_callback(move |drawing_area, clock| {
            if drawing_area.is_visible() {
                *frametime.borrow_mut() = clock.frame_time() as f64 / 1_000_000.0;
                drawing_area.queue_draw();
            }
            
            glib::ControlFlow::Continue
        });

        let primary_held = Rc::new(RefCell::new(false));
        drawing_area.add_controller(gesture::on_primary_down(clone!(
            #[strong] primary_held,
            #[strong] seek_position,
            #[strong] duration,
            #[weak] drawing_area,
            move |_, x, _| {
                *primary_held.borrow_mut() = true;

                let alloc = drawing_area.allocation();
                let rel_x = x.clamp(0.0, alloc.width() as f64);
                let new_pos = (rel_x / alloc.width() as f64) * (*duration.borrow());
                *seek_position.borrow_mut() = Some(new_pos);
                drawing_area.queue_draw();
            }
        )));

        drawing_area.add_controller(gesture::on_motion(clone!(
            #[strong] primary_held,
            #[strong] seek_position,
            #[strong] duration,
            #[weak] drawing_area,
            move |x, _| {
                if *primary_held.borrow() {
                    let alloc = drawing_area.allocation();
                    let rel_x = x.clamp(0.0, alloc.width() as f64);
                    let new_pos = (rel_x / alloc.width() as f64) * (*duration.borrow());
                    *seek_position.borrow_mut() = Some(new_pos);
                    drawing_area.queue_draw();
                }
            }
        )));

        drawing_area.add_controller(gesture::on_primary_up(clone!(
            #[strong] position,
            #[strong] duration,
            #[weak] drawing_area,
            move |_, _, _| {
                if *primary_held.borrow() {
                    let Some(player) = mpris::get_default_player() else {
                        return;
                    };

                    let Some(seek_pos) = *seek_position.borrow() else {
                        return;
                    };

                    let alloc = drawing_area.allocation();
                    let rel_x = ((seek_pos / *duration.borrow()) * alloc.width() as f64).clamp(0.0, alloc.width() as f64);
                    let old_pos = *position.borrow();
                    let new_pos = (rel_x / alloc.width() as f64) * (*duration.borrow());
                    let delta = new_pos - old_pos;
                    if let Err(e) = player.seek(delta as i64) {
                        error!(%e, "Error seeking MPRIS player");
                    }

                    *position.borrow_mut() = new_pos;
                    *seek_position.borrow_mut() = None;
                    drawing_area.queue_draw();
                    *primary_held.borrow_mut() = false;
                }
            }
        )));

        drawing_area.add_controller(gesture::on_vertical_scroll(|delta| {
            let Some(player) = mpris::get_default_player() else {
                return warn!("No MPRIS player available to seek");
            };

            let seek_amount = if delta < 0.0 {
                SEEK_STEP_MICROSECONDS
            } else {
                -SEEK_STEP_MICROSECONDS
            };

            if let Err(e) = player.seek(seek_amount) {
                error!(%e, "Failed to seek");
            }
        }));

        Self {
            drawing_area,
            position,
            duration,
        }
    }

    pub fn set_position(&self, position: f64) {
        *self.position.borrow_mut() = position;
        self.drawing_area.queue_draw();
    }

    pub fn set_duration(&self, duration: f64) {
        *self.duration.borrow_mut() = duration;
        self.drawing_area.queue_draw();
    }
}