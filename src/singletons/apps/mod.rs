pub mod pixbuf;

use std::{path::{Path, PathBuf}, sync::Mutex};
use freedesktop_desktop_entry::{default_paths, get_languages_from_env, Iter, DesktopEntry};
use notify::{event::{AccessKind, AccessMode}, EventKind, Watcher};
use once_cell::sync::Lazy;

use crate::{helpers::{matching, process}, SQL_CONNECTION};

pub struct WeightedDesktopEntry {
    pub entry: DesktopEntry,
    pub weight: usize,
}

pub static DESKTOPS: Lazy<Mutex<Vec<DesktopEntry>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub fn activate() {
    refresh_desktops();

    for path in default_paths() {
        std::thread::spawn(move || watch_desktops(&path));
    }
}

pub fn calculate_weight(entry: &DesktopEntry, query: &str) -> usize {
    // Fixed weights:
    // 1. Exact match (10000)
    // 2. Lazy match, contains all characters, length match (500)
    // 3. Fuzzy match (30)
    // 4. Lazy match (10)
    //
    // Dynamic weights:
    // 5. Beginning match (100)
    // 6. String inclusion bonus (query.length * 4)
    // 7. Launch count bonus (runs / 4, minimum 2x multiplier)
    let locales = get_languages_from_env();
    let name = entry.name(&locales).map(|c| c.to_string()).unwrap_or_default().to_lowercase();
    let query = &query.trim().to_lowercase();

    let lazy_match = matching::lazy_match(&name, query);
    let contains_all = name.chars().all(|c| query.contains(c));

    let mut weight = if name == *query {
        10000
    } else if lazy_match && contains_all && query.len() == name.len() {
        500
    } else if matching::fuzzy_match(&name, query) {
        30
    } else if lazy_match {
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

    // How many times has this entry been run?
    if weight > 0 {
        if let Some(sqlite) = SQL_CONNECTION.get() {
            if let Ok(runs) = sqlite.get_runs(entry.exec().unwrap_or_default()) {
                if runs > 0 {
                    weight *= std::cmp::max(2, (runs / 4) as usize);
                }
            }
        }
    }

    weight
}

pub fn get_from_command(command: &str) -> Option<DesktopEntry> {
    let desktops = DESKTOPS.lock().unwrap();

    for entry in desktops.iter() {
        if entry.exec() == Some(command) {
            return Some(entry.clone());
        }
    }

    None
}

pub fn query_desktops(query: &str) -> Vec<WeightedDesktopEntry> {
    let desktops = DESKTOPS.lock().unwrap();

    let mut weighted = desktops.iter()
        .map(|entry| WeightedDesktopEntry {
            entry: entry.clone(),
            weight: calculate_weight(entry, query)
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

/// This is a function that will invoke process::launch and tell SQLite to
/// increment the runs count for a desktop entry. Use this for launch tracking.
pub fn launch_and_track(command: &str) {
    process::launch(command);

    if let Some(sqlite) = SQL_CONNECTION.get() {
        let _ = sqlite.increment_runs(command);
    }
}

pub fn watch_desktops(path: &PathBuf) {
    let (tx, rx) = std::sync::mpsc::channel();
    
    let mut watcher = notify::recommended_watcher(tx).unwrap();
    let result = watcher.watch(
        Path::new(path),
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