mod minimal;
mod extended;
mod progress;

use gtk4::prelude::*;

use crate::singletons::mpris;
use crate::utils::gesture;
use super::super::module::{BarModule, BarModuleWrapper};

const VOLUME_STEP: f64 = 0.05;

pub fn new() -> BarModuleWrapper {
    let module = BarModule::new(minimal::minimal(), extended::extended());
    let wrapper = BarModuleWrapper::new(module, &["bar-mpris"]);

    wrapper.bx.add_controller({
        let module = wrapper.module.clone();
        gesture::on_middle_down(move |_, _, _| if !module.is_expanded() {
            let Some(player) = mpris::get_default_player() else {
                return eprintln!("No MPRIS player available to toggle play/pause.");
            };

            if let Err(e) = player.play_pause() {
                eprintln!("Failed to toggle play/pause: {}", e);
            }
        })
    });

    wrapper.bx.add_controller({
        let module = wrapper.module.clone();
        gesture::on_secondary_down(move |_, _, _| if !module.is_expanded() {
            mpris::with_default_player_mut(|player| if let Err(e) = player.next() {
                eprintln!("Failed to skip to next track: {}", e);
            });
        })
    });

    wrapper.bx.add_controller({
        let module = wrapper.module.clone();
        gesture::on_vertical_scroll(move |delta_y| if !module.is_expanded() {
            let Some(player) = mpris::get_default_player() else {
                return eprintln!("No MPRIS player available to adjust volume.");
            };

            let step = if delta_y < 0.0 {
                VOLUME_STEP
            } else {
                -VOLUME_STEP
            };

            player.adjust_volume(step)
                .unwrap_or_else(|e| eprintln!("Failed to adjust volume: {}", e));
        })
    });

    wrapper
}