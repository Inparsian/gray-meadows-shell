#[macro_use(view)]
extern crate relm4_macros;
#[macro_use]
mod macros;

mod color;
mod ipc;
mod ffi;
mod singletons;
mod widgets;
mod sql;
mod dbus;
mod display;
mod filesystem;
mod gesture;
mod matching;
mod process;
mod scss;
mod unit;
mod pixbuf;
mod timeout;
mod broadcast;
mod config;
mod session;

use std::{cell::RefCell, collections::HashMap};
use std::sync::{LazyLock, Mutex, OnceLock};
use futures_signals::signal::Mutable;
use gtk4::prelude::*;
use libadwaita::Application;
use sqlite::Connection;

use crate::widgets::{bar, windows::{self, GmsWindow}};

const FLOAT_TOLERANCE: f64 = 0.0001;

pub struct GrayMeadowsLocal {
    provider: gtk4::CssProvider,
    icon_theme: gtk4::IconTheme,
    pub bars: RefCell<Vec<widgets::bar::BarWindow>>,
    pub osd_containers: RefCell<Vec<widgets::osd::OsdWindow>>,
    pub windows: RefCell<HashMap<String, Box<dyn GmsWindow>>>,
}

thread_local! {
    pub static APP_LOCAL: RefCell<GrayMeadowsLocal> = RefCell::new(GrayMeadowsLocal {
        provider: gtk4::CssProvider::new(),
        icon_theme: gtk4::IconTheme::default(),
        bars: RefCell::new(Vec::new()),
        osd_containers: RefCell::new(Vec::new()),
        windows: RefCell::new(HashMap::new()),
    });
}

#[derive(Debug, Clone)]
pub struct GrayMeadowsGlobal {
    #[allow(dead_code)]
    config: config::Config,
    game_mode: Mutable<bool>,
}

pub static APP: LazyLock<GrayMeadowsGlobal> = LazyLock::new(|| GrayMeadowsGlobal {
    config: config::read().expect("Failed to read configuration"),
    game_mode: Mutable::new(false),
});

pub static SQL_CONNECTION: OnceLock<Mutex<Connection>> = OnceLock::new();

fn activate(application: &Application) {
    let keybinds_osd = widgets::osd::imp::keybinds::KeybindsOsd::default();
    let volume_osd = widgets::osd::imp::volume::VolumeOsd::default();

    for monitor in display::get_all_monitors(&gdk4::Display::default().expect("Failed to get default display")) {
        let bar = widgets::bar::BarWindow::new(application, &monitor);
        let osd = widgets::osd::OsdWindow::new(application, &monitor);

        osd.add_osd(&keybinds_osd);
        osd.add_osd(&volume_osd);

        bar.window.show();
        osd.window.show();

        APP_LOCAL.with(|app| {
            let app = app.borrow();
            app.bars.borrow_mut().push(bar);
            app.osd_containers.borrow_mut().push(osd);
        });
    }

    APP_LOCAL.with(|app| {
        app.borrow().windows.borrow_mut().insert("overview".into(), Box::new(widgets::windows::overview::new(application)));
        app.borrow().windows.borrow_mut().insert("session".into(), Box::new(widgets::windows::session::new(application)));
        app.borrow().windows.borrow_mut().insert("left_sidebar".into(), Box::new(widgets::windows::sidebar_left::new(application)));
        app.borrow().windows.borrow_mut().insert("right_sidebar".into(), Box::new(widgets::windows::sidebar_right::new(application)));

        // optional features
        if process::is_command_available("cliphist") && process::is_command_available("wl-copy") {
            app.borrow().windows.borrow_mut().insert("clipboard".into(), Box::new(widgets::windows::clipboard::new(application)));
        } else {
            println!("cliphist or wl-copy not found, clipboard window will not be available");
        }
    });
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    // If no arguments are provided, assume that the user wants to run the shell.
    // Otherwise, interpret the arguments as an IPC command.
    if args.len() == 1 {
        // Ensure that another instance of gray-meadows-shell is not running.
        if ipc::client::get_stream().is_ok() {
            eprintln!("Another instance of gray-meadows-shell is already running.");
            std::process::exit(1);
        } else {
            tokio::spawn(async {
                if let Err(e) = ipc::server::start().await {
                    eprintln!("Failed to start IPC server: {}", e);
                    std::process::exit(1);
                }
            });

            match sql::establish_connection() {
                Ok(connection) => {
                    let _ = SQL_CONNECTION.set(Mutex::new(connection));
                    println!("SQLite connection established successfully, storing data in {}/sqlite.db", filesystem::get_config_directory());
                }
                Err(e) => {
                    eprintln!("Failed to establish SQLite connection: {:?}", e);
                    std::process::exit(1);
                }
            }

            let _ = gtk4::init();

            gtk4::style_context_add_provider_for_display(
                &gdk4::Display::default().expect("Failed to get default display"),
                &APP_LOCAL.with(|app| app.borrow().provider.clone()),
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );

            if let Some(settings) = gtk4::Settings::default() {
                let current_icon_theme = settings.property::<String>("gtk-icon-theme-name");
                APP_LOCAL.with(|app| {
                    let icon_theme = &app.borrow().icon_theme;
                    icon_theme.set_theme_name(Some(&current_icon_theme));
                });
            }

            scss::bundle_apply_scss();
            scss::watch_scss();

            singletons::activate_all();
            windows::listen_for_ipc_messages();
            bar::listen_for_ipc_messages();

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
            |response| println!("Response from IPC server: {}", response)
        );
    }
}