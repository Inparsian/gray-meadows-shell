use std::{path::{Path, PathBuf}, sync::Mutex};
use freedesktop_desktop_entry::{default_paths, get_languages_from_env, Iter, DesktopEntry};
use notify::{event::{AccessKind, AccessMode}, EventKind, Watcher};
use once_cell::sync::Lazy;

use crate::helpers::matching;

pub struct WeightedDesktopEntry {
    pub entry: DesktopEntry,
    weight: usize,
}

pub static DESKTOPS: Lazy<Mutex<Vec<DesktopEntry>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub fn activate() {
    refresh_desktops();

    for path in default_paths() {
        std::thread::spawn(|| watch_desktops(path));
    }
}

pub fn query_desktops(query: &str) -> Vec<WeightedDesktopEntry> {
    let desktops = DESKTOPS.lock().unwrap();
    let locales = get_languages_from_env();

    let mut weighted = desktops.iter()
        .map(|entry| {
            // Fixed weights:
            // 1. Exact match (10000)
            // 2. Lazy match, contains all characters, length match (500)
            // 3. Fuzzy match (30)
            // 4. Lazy match (10)
            //
            // Dynamic weights:
            // 5. Beginning match (100)
            // 6. String inclusion bonus (query.length * 4)
            let name = entry.name(&locales).map(|c| c.to_string()).unwrap_or_default().to_lowercase();
            let query = &query.trim().to_lowercase();
            let mut weight = if name == *query {
                10000
            } else if matching::lazy_match(&name, query) {
                500
            } else if matching::fuzzy_match(&name, query) {
                30
            } else if matching::lazy_match(&name, query) {
                10
            } else {
                0
            };

            if name.starts_with(query) {
                weight += 100;
            }

            if name.contains(query) {
                weight += query.len() * 4;
            }

            WeightedDesktopEntry {
                entry: entry.clone(),
                weight
            }
        })
        .filter(|entry| entry.weight > 0)
        .collect::<Vec<_>>();

    weighted.sort_by(|a, b| b.weight.cmp(&a.weight));

    weighted
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