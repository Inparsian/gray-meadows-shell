use std::sync::{RwLock, LazyLock};
use std::collections::HashMap;

use crate::sql::wrappers::commands::{self, DesktopRunsEntry};

static RUNS: LazyLock<RwLock<HashMap<String, DesktopRunsEntry>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

pub async fn init() {
    match commands::get_all_runs().await {
        Ok(all_runs) => {
            let mut runs_writer = RUNS.write().unwrap();
            for run in all_runs {
                runs_writer.insert(run.command.clone(), run);
            }
        },
        
        Err(err) => {
            error!(%err, "Could not get all desktop runs");
        },
    }
}

pub fn get_entry(command: &str) -> Option<DesktopRunsEntry> {
    RUNS.read().unwrap().get(command).cloned()
}

pub fn increment_entry_runs(command: &str) {
    let now = chrono::Local::now();
    let mut writer = RUNS.write().unwrap();
    if let Some(entry) = writer.get_mut(command) {
        entry.last_run = now;
        entry.runs += 1;
    }
}

pub fn get_top_commands() -> Vec<DesktopRunsEntry> {
    let runs_guard = RUNS.read().unwrap();
    let mut runs_vec: Vec<DesktopRunsEntry> = runs_guard.values().cloned().collect();

    runs_vec.sort_by(|a, b| b.runs.cmp(&a.runs));
    runs_vec
}

pub fn get_most_recent_commands() -> Vec<DesktopRunsEntry> {
    let runs_guard = RUNS.read().unwrap();
    let mut runs_vec: Vec<DesktopRunsEntry> = runs_guard.values().cloned().collect();

    runs_vec.sort_by_key(|b| std::cmp::Reverse(b.last_run));
    runs_vec
}