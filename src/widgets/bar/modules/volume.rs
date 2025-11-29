use gtk4::prelude::*;

use crate::{
    ffi::astalwp::{WpEvent, ffi},
    helpers::gesture,
    window::PopupWindow,
    singletons::wireplumber,
    widgets::bar::wrapper::BarModuleWrapper
};

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

pub fn new() -> gtk4::Box {
    view! {
        volume_scroll_gesture = gesture::on_vertical_scroll(|delta_y| {
            if let Some(default_speaker) = wireplumber::get_default_speaker() {
                let new_volume = (delta_y as f32).mul_add(-VOLUME_STEP, default_speaker.node.volume).clamp(0.0, 1.5);

                ffi::node_set_volume(default_speaker.node.id, new_volume);
            }
        }),

        volume_click_gesture = gesture::on_primary_down(|_, _, _| {
            PopupWindow::SidebarRight.toggle();
        }),

        volume_char_label = gtk4::Label {
            set_label: LOW_VOLUME_CHAR,
        },

        volume_percentage_label = gtk4::Label {
            set_label: "0%",
        },

        widget = gtk4::Box {
            set_css_classes: &["bar-widget"],
            set_spacing: 6,
            set_hexpand: false,

            append: &volume_char_label,
            append: &volume_percentage_label
        }
    }

    // We can't initialize the volume immediately because the WirePlumber singleton
    // might not be ready yet. Subscribe to events.
    let (tx, rx) = async_channel::bounded::<f32>(1);
    tokio::spawn(async move {
        while let Ok(event) = wireplumber::subscribe().recv().await {
            match event {
                WpEvent::CreateSpeaker(endpoint) => {
                    if endpoint.is_default {
                        let _ = tx.send(endpoint.node.volume).await;
                    }
                },
            
                WpEvent::RemoveSpeaker(endpoint) => {
                    if let Some(default_speaker) = wireplumber::get_default_speaker() {
                        if default_speaker.node.id == endpoint.node.id {
                            let _ = tx.send(default_speaker.node.volume).await;
                        }
                    }
                },
            
                WpEvent::UpdateDefaultSpeaker(id) => {
                    if let Some(speaker) = wireplumber::get_endpoint(id) {
                        let _ = tx.send(speaker.node.volume).await;
                    }
                },
            
                WpEvent::UpdateEndpoint(id, property_name) => if property_name == "volume" {
                    if let Some(default_speaker) = wireplumber::get_default_speaker() {
                        if default_speaker.node.id == id {
                            let _ = tx.send(default_speaker.node.volume).await;
                        }
                    }
                },
            
                _ => {}
            }
        }
    });

    gtk4::glib::spawn_future_local(async move {
        while let Ok(volume) = rx.recv().await {
            volume_char_label.set_label(&volume_to_char(volume));
            volume_percentage_label.set_label(&format!("{:.0}%", volume * 100.0));
        }
    });

    BarModuleWrapper::new(&widget)
        .add_controller(volume_scroll_gesture)
        .add_controller(volume_click_gesture)
        .get_widget()
}