use futures_signals::signal::SignalExt;
use ::hyprland::dispatch;
use ::hyprland::instance::Instance;

use crate::{
    widgets::sidebar_right::quicktoggle::{QuickToggle, QuickToggleMuiIcon},
    singletons::hyprland,
};

pub fn new() -> gtk4::Button {
    let toggle = QuickToggle::new_from_icon(
        QuickToggleMuiIcon::new("keyboard_off", "keyboard"),
        Some(Box::new(|current_state| {
            if let Ok(instance) = Instance::from_current_env() {
                let submap = if current_state { "" } else { "grab" };

                let _ = dispatch!(&instance, Custom, "submap", submap);
            }

            !current_state
        })),
    );

    let button = toggle.button.clone();

    gtk4::glib::spawn_future_local(signal_cloned!(hyprland::HYPRLAND.submap, (submap) {
        if let Some(submap) = submap {
            toggle.set_toggled(submap == "grab");
        } else {
            // Assume that the absence of a submap means that the grab submap is not active
            toggle.set_toggled(false);
        }
    }));

    button
}