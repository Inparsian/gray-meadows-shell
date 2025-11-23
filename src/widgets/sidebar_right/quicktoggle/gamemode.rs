use futures_signals::signal::SignalExt;
use ::hyprland::ctl::reload;

use crate::{
    APP, singletons::hyprland::{call_hyprctl, call_hyprctl_batch}, widgets::sidebar_right::quicktoggle::{QuickToggle, QuickToggleMuiIcon}
};

pub fn new() -> gtk4::Button {
    let toggle = QuickToggle::new_from_icon(
        QuickToggleMuiIcon::new("gamepad", "gamepad"),
        Some(Box::new(|_| {
            let new_state = if APP.game_mode.get() {
                let _ = reload::call();
                false
            } else {
                let keywords = [
                    "keyword windowrule immediate 1, fullscreenstate:* 1",
                    "keyword windowrule bordersize 0, fullscreenstate:* 1",
                    "keyword animations:enabled 0",
                    "keyword decoration:shadow:enabled 0",
                    "keyword decoration:blur:enabled 0",
                    "keyword general:gaps_in 0",
                    "keyword general:gaps_out 0",
                    "keyword general:border_size 1",
                    "keyword decoration:rounding 0",
                    "keyword general:allow_tearing 1"
                ];

                call_hyprctl_batch(&keywords);
                true
            };
            
            APP.game_mode.set(new_state);

            // This is an edge case in case game_mode somehow becomes desynced with hyprland's animation toggle
            gtk4::glib::spawn_future_local(async move {
                gtk4::glib::timeout_future(std::time::Duration::from_millis(100)).await;

                if let Some(message) = call_hyprctl("getoption animations:enabled") {
                    let hyprland_state = message.split('\n').next()
                        .and_then(|line| line.split("int: ").nth(1))
                        .is_some_and(|val| val == "0");

                    if hyprland_state != APP.game_mode.get() {
                        APP.game_mode.set(hyprland_state);
                    }
                }
            });

            new_state
        })),
    );

    let button = toggle.button.clone();

    gtk4::glib::spawn_future_local(signal!(APP.game_mode, (game_mode) {
        toggle.set_toggled(game_mode);
    }));

    button
}