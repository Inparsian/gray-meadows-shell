mod minimal;
mod extended;
mod progress;

use gtk4::prelude::*;

use crate::services::mpris;
use crate::utils::gesture;
use super::super::base::BarModule;

const VOLUME_STEP: f64 = 0.05;

pub fn new() -> BarModule {
    let module = BarModule::with_widgets(&minimal::minimal().upcast(), &extended::extended().upcast());
    module.add_css_class("bar-mpris");

    module.add_controller(gesture::on_middle_down(clone!(
        #[weak] module,
        move |_, _, _| if !module.expanded() {
            let Some(player) = mpris::get_default_player() else {
                return warn!("No MPRIS player available to toggle play/pause");
            };

            if let Err(e) = player.play_pause() {
                error!(%e, "Failed to toggle play/pause");
            }
        }
    )));

    module.add_controller(gesture::on_secondary_down(clone!(
        #[weak] module,
        move |_, _, _| if !module.expanded() {
            mpris::with_default_player_mut(|player| if let Err(e) = player.next() {
                error!(%e, "Failed to skip to next track");
            });
        }
    )));

    module.add_controller(gesture::on_vertical_scroll(clone!(
        #[weak] module,
        move |delta_y| if !module.expanded() {
            let Some(player) = mpris::get_default_player() else {
                return warn!("No MPRIS player available to adjust volume");
            };

            let step = if delta_y < 0.0 {
                VOLUME_STEP
            } else {
                -VOLUME_STEP
            };

            player.adjust_volume(step)
                .unwrap_or_else(|e| error!(%e, "Failed to adjust volume"));
        }
    )));

    module
}