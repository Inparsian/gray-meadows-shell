use gtk::prelude::*;

use crate::services::screen_recorder::{ScreenRecorderState, get_screen_recorder};
use super::super::base::BarModule;
use futures_signals::signal::SignalExt as _;

const REPLAY_CHAR: &str = "○";
const RECORDING_CHAR: &str = "⏺";

pub fn new() -> BarModule {
    view! {
        replay_char_label = gtk::Label {
            set_css_classes: &["screen-recorder-indicator", "replay"],
            set_label: REPLAY_CHAR,
            set_visible: false,
        },

        recording_char_label = gtk::Label {
            set_css_classes: &["screen-recorder-indicator", "recording"],
            set_label: RECORDING_CHAR,
            set_visible: false,
        },

        widget = gtk::Box {
            set_spacing: 0,
            set_hexpand: false,
            
            append: &replay_char_label,
            append: &recording_char_label
        }
    }

    let module = BarModule::builder()
        .minimal_widget(&widget)
        .build();

    // start off invis
    module.set_visible(false);

    if let Ok(screen_recorder) = get_screen_recorder().read() {
        glib::spawn_future_local({
            let module = module.downgrade();
            signal!(screen_recorder.state, (new_state) {
                if let Some(module) = module.upgrade() {
                    replay_char_label.set_visible(matches!(new_state, ScreenRecorderState::Replay));
                    recording_char_label.set_visible(matches!(new_state, ScreenRecorderState::Record));

                    // i.e. if it's *neither* of them, we need to hide this module
                    module.set_visible(matches!(new_state, ScreenRecorderState::Record | ScreenRecorderState::Replay));
                }
            })
        });
    }
    
    module
}