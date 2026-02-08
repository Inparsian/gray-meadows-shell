#[macro_use(clone, closure_local)]
extern crate glib;
#[macro_use(view)]
extern crate relm4_macros;
#[macro_use(debug, info, warn, error)]
extern crate tracing;
#[macro_use]
mod macros;

mod color;
mod ipc;
mod ffi;
mod services;
mod widgets;
mod sql;
mod utils;
mod dbus;
mod scss;
mod pixbuf;
mod config;
mod session;

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::LazyLock;
use futures_signals::signal::Mutable;
use gtk::prelude::*;
use libadwaita::Application;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::time::ChronoLocal;

use self::sql::actor::SqlActor;
use self::widgets::{bar, windows::{self, GmsWindow}, notifications::NotificationsWindow};
use self::utils::{display, process};

const FLOAT_TOLERANCE: f64 = 0.0001;

pub struct GrayMeadowsLocal {
    provider: gtk::CssProvider,
    icon_theme: gtk::IconTheme,
    pub bars: RefCell<Vec<widgets::bar::BarWindow>>,
    pub osd_containers: RefCell<Vec<widgets::osd::OsdWindow>>,
    pub notification_containers: RefCell<Vec<NotificationsWindow>>,
    pub windows: RefCell<HashMap<String, Box<dyn GmsWindow>>>,
}

thread_local! {
    pub static APP_LOCAL: GrayMeadowsLocal = GrayMeadowsLocal {
        provider: gtk::CssProvider::new(),
        icon_theme: gtk::IconTheme::default(),
        bars: RefCell::new(Vec::new()),
        osd_containers: RefCell::new(Vec::new()),
        notification_containers: RefCell::new(Vec::new()),
        windows: RefCell::new(HashMap::new()),
    };
}

#[derive(Debug, Clone)]
pub struct GrayMeadowsGlobal {
    game_mode: Mutable<bool>,
    do_not_disturb: Mutable<bool>,
}

pub static APP: LazyLock<GrayMeadowsGlobal> = LazyLock::new(|| GrayMeadowsGlobal {
    game_mode: Mutable::new(false),
    do_not_disturb: Mutable::new(false),
});

pub static SQL_ACTOR: LazyLock<SqlActor> = LazyLock::new(SqlActor::default);

pub static USERNAME: LazyLock<String> = LazyLock::new(|| {
    std::env::var("USER").unwrap_or_else(|_| "unknown".to_owned())
});

fn activate(application: &Application) {
    let keybinds_osd = widgets::osd::imp::keybinds::KeybindsOsd::default();
    let volume_osd = widgets::osd::imp::volume::VolumeOsd::default();

    for monitor in display::get_all_monitors(&gdk::Display::default().expect("Failed to get default display")) {
        let bar = widgets::bar::BarWindow::new(application, &monitor);
        let osd = widgets::osd::OsdWindow::new(application, &monitor);
        let notifications_window = NotificationsWindow::new(application, &monitor);

        osd.add_osd(&keybinds_osd);
        osd.add_osd(&volume_osd);

        bar.window.show();
        osd.window.show();
        notifications_window.window.show();

        APP_LOCAL.with(|app| {
            app.bars.borrow_mut().push(bar);
            app.osd_containers.borrow_mut().push(osd);
            app.notification_containers.borrow_mut().push(notifications_window);
        });
    }

    widgets::notifications::listen_for_notifications();

    APP_LOCAL.with(|app| {
        let mut windows = app.windows.borrow_mut();
        windows.insert("overview".into(), Box::new(widgets::windows::overview::new(application)));
        windows.insert("session".into(), Box::new(widgets::windows::session::new(application)));
        windows.insert("left_sidebar".into(), Box::new(widgets::windows::sidebar_left::new(application)));
        windows.insert("right_sidebar".into(), Box::new(widgets::windows::sidebar_right::new(application)));

        // optional features
        if process::is_command_available("cliphist") && process::is_command_available("wl-copy") {
            windows.insert("clipboard".into(), Box::new(widgets::windows::clipboard::new(application)));
        } else {
            warn!("cliphist or wl-copy not found, clipboard window will not be available");
        }
    });
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    // If no arguments are provided, assume that the user wants to run the shell.
    // Otherwise, interpret the arguments as an IPC command.
    if args.len() == 1 {
        tracing_subscriber::fmt()
            .with_line_number(true)
            .with_env_filter(EnvFilter::from_default_env())
            .with_timer(ChronoLocal::new("%H:%M:%S%.3f".to_owned()))
            .init();

        // Ensure that another instance of gray-meadows-shell is not running.
        if ipc::client::get_stream().is_ok() {
            error!("Another instance of gray-meadows-shell is already running.");
            std::process::exit(1);
        } else {
            tokio::spawn(async {
                if let Err(e) = ipc::server::start().await {
                    error!(%e, "Failed to start IPC server");
                    std::process::exit(1);
                }
            });

            sql::init_database().await;
            APP.do_not_disturb.set(sql::wrappers::state::get_do_not_disturb().await.unwrap_or(false));

            let _ = gtk::init();

            gtk::style_context_add_provider_for_display(
                &gdk::Display::default().expect("Failed to get default display"),
                &APP_LOCAL.with(|app| app.provider.clone()),
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );

            if let Some(settings) = gtk::Settings::default() {
                let current_icon_theme = settings.property::<String>("gtk-icon-theme-name");
                APP_LOCAL.with(|app| {
                    app.icon_theme.set_theme_name(Some(&current_icon_theme));
                });
            }

            scss::bundle_apply_scss();
            scss::watch_scss();

            services::activate_all().await;
            windows::listen_for_ipc_messages();
            bar::listen_for_ipc_messages();
            config::watch();

            let application = Application::new(
                Some("sn.inpr.gray_meadows_shell"),
                Default::default(),
            );

            application.connect_activate(activate);
            application.run();
        }
    } else {
        let command = args[1..].join(" ");

        ipc::client::send_message(&command).map_or_else(
            |err| eprintln!("Failed to send IPC command: {}", err),
            |response| println!("{}", response)
        );
    }
}