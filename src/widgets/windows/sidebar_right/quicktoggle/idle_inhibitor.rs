use futures_signals::signal::SignalExt as _;
use gtk::prelude::*;

use crate::services::idle_inhibitor::IDLE_INHIBITOR;
use crate::sql::wrappers::state::set_idle_inhibited;
use super::{QuickToggle, QuickToggleMuiIcon};

pub fn new() -> gtk::Button {
    let toggle = QuickToggle::new_from_icon(
        QuickToggleMuiIcon::new("coffee", "coffee"),
        Some(Box::new(|current_state| {
            IDLE_INHIBITOR.get().is_some_and(|inhibitor| {
                inhibitor.set_inhibited(!current_state);
                !current_state
            })
        })),
    );

    let button = toggle.button.clone();

    if let Some(inhibitor) = IDLE_INHIBITOR.get() {
        glib::spawn_future_local(signal!(inhibitor.inhibited, (inhibited) {
            toggle.set_toggled(inhibited);

            glib::spawn_future_local(async move {
                if let Err(err) = set_idle_inhibited(inhibited).await {
                    error!(%err, "Failed to set idle inhibited state");
                }
            });
        }));
    } else {
        button.set_sensitive(false);
    }

    button
}
