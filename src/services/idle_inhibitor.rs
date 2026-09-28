use std::sync::{Mutex, OnceLock};
use futures_signals::signal::Mutable;
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_client::protocol::wl_compositor::WlCompositor;
use wayland_client::protocol::wl_registry::{self, WlRegistry};
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_protocols::wp::idle_inhibit::zv1::client::zwp_idle_inhibit_manager_v1::ZwpIdleInhibitManagerV1;
use wayland_protocols::wp::idle_inhibit::zv1::client::zwp_idle_inhibitor_v1::ZwpIdleInhibitorV1;

use crate::sql::wrappers::state::get_idle_inhibited;

pub struct IdleInhibitor {
    conn: Connection,
    qh: QueueHandle<State>,
    surface: WlSurface,
    manager: ZwpIdleInhibitManagerV1,
    inhibitor: Mutex<Option<ZwpIdleInhibitorV1>>,
    pub inhibited: Mutable<bool>,
}

// Remains unset if the compositor does not support idle_inhibit_unstable_v1
pub static IDLE_INHIBITOR: OnceLock<IdleInhibitor> = OnceLock::new();

#[derive(Default)]
struct State {
    compositor: Option<WlCompositor>,
    manager: Option<ZwpIdleInhibitManagerV1>,
}

impl Dispatch<WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: wl_registry::Event,
        (): &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global { name, interface, version } = event {
            match interface.as_str() {
                "wl_compositor" => {
                    state.compositor = Some(registry.bind(name, version.min(6), qh, ()));
                },

                "zwp_idle_inhibit_manager_v1" => {
                    state.manager = Some(registry.bind(name, version.min(1), qh, ()));
                },

                _ => {}
            }
        }
    }
}

wayland_client::delegate_noop!(State: ignore WlCompositor);
wayland_client::delegate_noop!(State: ignore WlSurface);
wayland_client::delegate_noop!(State: ignore ZwpIdleInhibitManagerV1);
wayland_client::delegate_noop!(State: ignore ZwpIdleInhibitorV1);

impl IdleInhibitor {
    pub fn set_inhibited(&self, inhibited: bool) {
        let mut inhibitor = self.inhibitor.lock().unwrap();

        if inhibited {
            if inhibitor.is_none() {
                *inhibitor = Some(self.manager.create_inhibitor(&self.surface, &self.qh, ()));
            }
        } else if let Some(inhibitor) = inhibitor.take() {
            inhibitor.destroy();
        }

        if let Err(err) = self.conn.flush() {
            error!(%err, "Failed to flush wayland connection");
        }

        self.inhibited.set(inhibited);
    }
}

pub async fn activate() {
    let conn = match Connection::connect_to_env() {
        Ok(conn) => conn,
        Err(err) => {
            error!(%err, "Failed to connect to wayland display, idle inhibitor will not be available");
            return;
        }
    };

    let mut event_queue = conn.new_event_queue::<State>();
    let qh = event_queue.handle();
    let mut state = State::default();

    conn.display().get_registry(&qh, ());

    if let Err(err) = event_queue.roundtrip(&mut state) {
        error!(%err, "Failed to roundtrip wayland connection, idle inhibitor will not be available");
        return;
    }

    let (Some(compositor), Some(manager)) = (state.compositor.clone(), state.manager.clone()) else {
        warn!("Compositor does not support idle_inhibit_unstable_v1, idle inhibitor will not be available");
        return;
    };

    let surface = compositor.create_surface(&qh, ());

    let inhibitor = IDLE_INHIBITOR.get_or_init(|| IdleInhibitor {
        conn,
        qh,
        surface,
        manager,
        inhibitor: Mutex::new(None),
        inhibited: Mutable::new(false),
    });

    // Keep draining events from this connection so the socket buffer never fills up
    std::thread::spawn(move || loop {
        if let Err(err) = event_queue.blocking_dispatch(&mut state) {
            error!(%err, "Idle inhibitor wayland connection dispatch failed");
            break;
        }
    });

    match get_idle_inhibited().await {
        Ok(true) => inhibitor.set_inhibited(true),
        Ok(false) => {},
        Err(err) => error!(%err, "Failed to restore idle inhibitor state"),
    }
}
