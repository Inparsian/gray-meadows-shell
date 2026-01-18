use futures_signals::signal::SignalExt as _;

use crate::APP;
use crate::sql::wrappers::state::set_do_not_disturb;
use super::{QuickToggle, QuickToggleMuiIcon};

pub fn new() -> gtk4::Button {
    let toggle = QuickToggle::new_from_icon(
        QuickToggleMuiIcon::new("notifications_off", "notifications_active"),
        Some(Box::new(|_| {
            let new_state = !APP.do_not_disturb.get();
            APP.do_not_disturb.set(new_state);
            new_state
        })),
    );

    let button = toggle.button.clone();

    gtk4::glib::spawn_future_local(signal!(APP.do_not_disturb, (do_not_disturb) {
        toggle.set_toggled(do_not_disturb);
        
        gtk4::glib::spawn_future_local(async move {
            if let Err(err) = set_do_not_disturb(do_not_disturb).await {
                error!(%err, "Failed to set do not disturb");
            }
        });
    }));

    button
}