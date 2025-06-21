mod ffi;
mod helpers;
mod reactivity;
mod singletons;
mod widgets;

use gtk4::prelude::*;
use libadwaita::Application;
use notify::{event::AccessKind, Watcher};
use once_cell::sync::Lazy;
use std::{path::Path, sync::Mutex};

pub struct GrayMeadows {
    provider: gtk4::CssProvider,
}

unsafe impl Send for GrayMeadows {}

pub static APP: Lazy<Mutex<GrayMeadows>> = Lazy::new(|| {
    Mutex::new(GrayMeadows {
        provider: gtk4::CssProvider::new(),
    })
});

pub fn bundle_apply_scss() {
    gtk4::glib::MainContext::default().invoke(|| {
        let styles_path = helpers::cargo::get_styles_directory();
        
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
        let app = &APP.lock().unwrap();
        let css = std::fs::read_to_string(format!("{}/output.css", styles_path))
            .expect("Failed to read output.css");
    
        app.provider.load_from_data(&css); 
    });
}

fn watch_scss() {
    tokio::spawn(async move {
        let (tx, rx) = std::sync::mpsc::channel();
        let styles_path = helpers::cargo::get_styles_directory();

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
                        if event.paths.iter().any(|p| p.extension() == Some("scss".as_ref()) && matches!(event.kind, notify::EventKind::Access(AccessKind::Close(notify::event::AccessMode::Write)))) {
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
}

#[tokio::main]
async fn main() {
    let _ = gtk4::init();

    // Activate singletons
    singletons::date_time::activate();
    singletons::mpris::activate();

    // Add the CSS provider to the default display
    gtk4::style_context_add_provider_for_display(
        &gdk4::Display::default().expect("Failed to get default display"),
        &APP.lock().unwrap().provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // Bundle and apply the SCSS, then watch for changes
    bundle_apply_scss();
    watch_scss();

    // Initialize and run the application
    let application = Application::new(
        Some("sn.inpr.gray_meadows_shell"),
        Default::default(),
    );

    application.connect_activate(|app| {
        activate(app);
    });

    application.run();
}