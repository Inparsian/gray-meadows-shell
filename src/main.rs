#[macro_use(view)]
extern crate relm4_macros;
#[macro_use]
mod macros;

mod color;
mod ipc;
mod ffi;
mod helpers;
mod singletons;
mod widgets;
mod sql;
mod dbus;

use std::{cell::RefCell, path::Path, sync::{LazyLock, Mutex, OnceLock}};
use futures_signals::signal::Mutable;
use gtk4::prelude::*;
use libadwaita::Application;
use notify::{EventKind, event::{AccessKind, AccessMode}, Watcher};
use sqlite::Connection;

pub struct GrayMeadowsLocal {
    provider: gtk4::CssProvider,
    icon_theme: gtk4::IconTheme
}

thread_local! {
    pub static APP_LOCAL: RefCell<GrayMeadowsLocal> = RefCell::new(GrayMeadowsLocal {
        provider: gtk4::CssProvider::new(),
        icon_theme: gtk4::IconTheme::default()
    });
}

#[derive(Debug, Clone)]
pub struct GrayMeadowsGlobal {
    game_mode: Mutable<bool>,
}

pub static APP: LazyLock<GrayMeadowsGlobal> = LazyLock::new(|| GrayMeadowsGlobal {
    game_mode: Mutable::new(false),
});

pub static SQL_CONNECTION: OnceLock<Mutex<Connection>> = OnceLock::new();

pub fn bundle_apply_scss() {
    gtk4::glib::MainContext::default().invoke(|| {
        let styles_path = helpers::filesystem::get_styles_directory();
        
        // Run sass
        let output = std::process::Command::new("sass")
            .arg(format!("-I {}", styles_path))
            .arg(format!("{}/main.scss", styles_path))
            .arg(format!("{}/output.css", styles_path))
            .output()
            .expect("Failed to run sass command");
        
        if !output.status.success() {
            eprintln!("Error running sass: {}", String::from_utf8_lossy(&output.stderr));
            return;
        }
    
        // Load the generated CSS into the provider
        let css = std::fs::read_to_string(format!("{}/output.css", styles_path))
            .expect("Failed to read output.css");

        APP_LOCAL.with(|app| app.borrow().provider.load_from_data(&css));
        
        // Refresh SCSS variables
        helpers::scss::refresh_variables();
    });
}

fn watch_scss() {
    tokio::spawn(async move {
        let (tx, rx) = std::sync::mpsc::channel();
        let styles_path = helpers::filesystem::get_styles_directory();

        let mut watcher = notify::recommended_watcher(tx).unwrap();
        let result = watcher.watch(
            Path::new(&styles_path),
            notify::RecursiveMode::Recursive,
        );

        if result.is_ok() {
            println!("Watching styles directory: {}", styles_path);

            for res in rx {
                match res {
                    Ok(event) => {
                        // If the event kind is Access(Close(Write)), it means the file is done being written to
                        if event.paths.iter().any(|p| p.extension() == Some("scss".as_ref()) && matches!(event.kind, EventKind::Access(AccessKind::Close(AccessMode::Write)))) {
                            println!("Styles changed: {:?}", event.paths);
                            bundle_apply_scss();
                        }
                    },

                    Err(e) => {
                        eprintln!("Error watching styles directory: {}", e);
                    }
                }
            }
        } else {
            eprintln!("Failed to watch styles directory: {}", result.unwrap_err());
        }
    });
}

fn activate(application: &Application) {
    for monitor in helpers::display::get_all_monitors(&gdk4::Display::default().expect("Failed to get default display")) {
        let bar = widgets::bar::Bar::new(application, &monitor);
        bar.window.show();
    }

    widgets::overview::new(application);
    widgets::session::new(application);
    widgets::sidebar_left::new(application);
    widgets::sidebar_right::new(application);
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
            std::thread::spawn(|| {
                if let Err(e) = ipc::server::start() {
                    eprintln!("Failed to start IPC server: {}", e);
                    std::process::exit(1);
                }
            });

            match sql::establish_connection() {
                Ok(connection) => {
                    let _ = SQL_CONNECTION.set(Mutex::new(connection));
                    println!("SQLite connection established successfully, storing data in {}/sqlite.db", helpers::filesystem::get_config_directory());
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

            // Add your manual search paths here
            // TODO: Replace this with an automatic search for the currently equipped icon theme
            APP_LOCAL.with(|app| {
                let icon_theme = &app.borrow().icon_theme;
                icon_theme.add_search_path(Path::new("/home/inparsian/.icons/besgnulinux-mono-grey/apps/scalable"));
                icon_theme.set_theme_name(Some("besgnulinux-mono-grey"));
            });

            bundle_apply_scss();
            watch_scss();

            singletons::activate_all();

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