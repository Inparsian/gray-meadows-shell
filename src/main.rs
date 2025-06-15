mod ffi;
mod helpers;
mod reactivity;
mod singletons;
mod widgets;

use gtk4::prelude::*;
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

fn activate(application: &gtk4::Application) {
    // Get all monitors
    for monitor in helpers::display::get_all_monitors(&gdk4::Display::default().expect("Failed to get default display")) {
        // Create a new bar for each monitor
        let bar = widgets::bar::Bar::new(application, &monitor);
        bar.window.show();
    }
}

#[tokio::main]
async fn main() {
    let _ = gtk4::init();

    // Activate singletons
    singletons::date_time::activate();

    // Add the CSS provider to the default display
    gtk4::style_context_add_provider_for_display(
        &gdk4::Display::default().expect("Failed to get default display"),
        &APP.lock().unwrap().provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // Bundle and apply the SCSS
    bundle_apply_scss();

    // Watch the styles directory for changes
    std::thread::spawn(|| {
        let styles_path = helpers::cargo::get_styles_directory();
        let (tx, rx) = std::sync::mpsc::channel();

        println!("Watching styles directory: {}", styles_path);

        let mut watcher = notify::recommended_watcher(
            move |res: Result<notify::Event, notify::Error>| tx.send(res).unwrap()
        ).unwrap();

        watcher.watch(
            Path::new(&styles_path),
            notify::RecursiveMode::Recursive,
        ).expect("Failed to watch styles directory");

        for res in rx {
            match res {
                Ok(event) => {
                    // If the event kind is Access(Close(Write)), it means the file is done being written to
                    if event.paths.iter().any(|p| p.extension() == Some("scss".as_ref()) && matches!(event.kind, notify::EventKind::Access(AccessKind::Close(notify::event::AccessMode::Write)))) {
                        println!("Styles changed: {:?}", event.paths);

                        // Yell at the main thread to reapply the styles
                        bundle_apply_scss();
                    }
                },

                Err(e) => {
                    eprintln!("Failed to watch desktop files: {}", e);
                }
            }
        }
    });

    // Initialize the application
    let application = gtk4::Application::new(
        Some("sn.inpr.gray_meadows_shell"),
        Default::default(),
    );

    application.connect_activate(|app| {
        activate(app);
    });

    // Run the application
    application.run();
}