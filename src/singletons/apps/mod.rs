use std::{path::{Path, PathBuf}, sync::Mutex};
use freedesktop_desktop_entry::{default_paths, get_languages_from_env, Iter, DesktopEntry};
use notify::{event::{AccessKind, AccessMode}, EventKind, Watcher};
use once_cell::sync::Lazy;

pub static DESKTOPS: Lazy<Mutex<Vec<DesktopEntry>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub fn activate() {
    // Perform initial load of .desktop files
    refresh_desktops();

    // Watch for changes in the default .desktop file directories
    for path in default_paths() {
        std::thread::spawn(|| watch_desktops(path));
    }
}

pub fn refresh_desktops() {
    let locales = get_languages_from_env();

    let entries = Iter::new(default_paths())
        .entries(Some(&locales))
        .collect::<Vec<DesktopEntry>>();

    let mut desktops = DESKTOPS.lock().unwrap();
    desktops.clear();

    for entry in entries {
        desktops.push(entry);
    }
}

pub fn watch_desktops(path: PathBuf) {
    let (tx, rx) = std::sync::mpsc::channel();
    
    let mut watcher = notify::recommended_watcher(tx).unwrap();
    let result = watcher.watch(
        Path::new(&path),
        notify::RecursiveMode::Recursive,
    );

    if result.is_ok() {
        println!("Watching .desktop files in directory: {}", path.to_string_lossy());

        for res in rx {
            match res {
                Ok(event) => {
                    if event.paths.iter().any(|p| p.extension() == Some("desktop".as_ref())) {
                        match event.kind {
                            EventKind::Create(_) | EventKind::Remove(_) => {
                                println!("Desktop file added/removed: {:?}", event.paths);
                                refresh_desktops();
                            },

                            EventKind::Access(AccessKind::Close(AccessMode::Write)) => {
                                println!("Desktop file written to: {:?}", event.paths);
                                refresh_desktops();
                            },

                            _ => {}
                        }
                    }
                },

                Err(e) => {
                    eprintln!("Error watching .desktop directory: {}", e);
                }
            }
        }
    } else {
        eprintln!("Failed to watch .desktop directory: {}", result.unwrap_err());
    }
}