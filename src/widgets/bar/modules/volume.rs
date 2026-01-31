use gtk4::prelude::*;

use crate::ffi::astalwp::ffi;
use crate::utils::gesture;
use crate::services::wireplumber;
use crate::widgets::windows;
use super::super::base::BarModule;

const VOLUME_STEP: f32 = 0.05;
const LOW_VOLUME_CHAR: &str = "";
const MID_VOLUME_CHAR: &str = "";
const HIGH_VOLUME_CHAR: &str = "";

fn volume_to_char(volume: f32) -> String {
    if volume > 0.6 {
        HIGH_VOLUME_CHAR.to_owned()
    } else if volume > 0.3 {
        MID_VOLUME_CHAR.to_owned()
    } else {
        LOW_VOLUME_CHAR.to_owned()
    }
}

pub fn new() -> BarModule {
    view! {
        volume_scroll_gesture = gesture::on_vertical_scroll(|delta_y| {
            if let Some(default_speaker) = wireplumber::get_default_speaker() {
                let new_volume = (delta_y as f32).mul_add(-VOLUME_STEP, default_speaker.node.volume).clamp(0.0, 1.5);

                ffi::node_set_volume(default_speaker.node.id, new_volume);
            }
        }),

        volume_click_gesture = gesture::on_primary_down(|_, _, _| {
            windows::toggle("right_sidebar");
        }),

        volume_char_label = gtk4::Label {
            set_label: LOW_VOLUME_CHAR,
        },

        volume_percentage_label = gtk4::Label {
            set_label: "0%",
        },

        widget = gtk4::Box {
            set_spacing: 6,
            set_hexpand: false,
            
            append: &volume_char_label,
            append: &volume_percentage_label
        }
    }

    wireplumber::subscribe_default_speaker_volume(move |volume: f32| {
        volume_char_label.set_label(&volume_to_char(volume));
        volume_percentage_label.set_label(&format!("{:.0}%", volume * 100.0));
    });

    let module = BarModule::builder()
        .minimal_widget(&widget.upcast())
        .build();
    module.add_controller(volume_scroll_gesture);
    module.add_controller(volume_click_gesture);
    module
}