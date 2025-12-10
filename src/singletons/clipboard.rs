use std::{collections::HashMap, process::Stdio};
use std::io::{Read, Write};
use std::sync::{LazyLock, Mutex};
use regex::Regex;

use crate::process;

static IMAGE_BINARY_DATA_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[\[ binary data (\d+) ([KMGT]i)?B (\w+) (\d+)x(\d+) \]\]").expect("Failed to compile image binary data regex")
});
static DECODE_CACHE: LazyLock<Mutex<HashMap<usize, Vec<u8>>>> = LazyLock::new(|| {
    Mutex::new(HashMap::new())
});

pub fn decode_clipboard_entry(id: &str) -> Option<Vec<u8>> {
    if DECODE_CACHE.lock().unwrap().contains_key(&id.parse::<usize>().ok()?) {
        return DECODE_CACHE.lock().unwrap().get(&id.parse::<usize>().ok()?).cloned();
    }

    // fetch the image data from cliphist
    let output = std::process::Command::new("cliphist")
        .arg("decode")
        .arg(id)
        .output()
        .ok()?;

    output.status.success().then_some(output.stdout)
        .inspect(|data| if let Ok(parsed_id) = id.parse::<usize>() {
            DECODE_CACHE.lock().unwrap().insert(parsed_id, data.clone());
        })
}

pub fn is_an_image_clipboard_entry(preview: &str) -> bool {
    IMAGE_BINARY_DATA_PATTERN.is_match(preview)
}

pub fn fetch_clipboard_entries() -> Vec<(usize, String)> {
    if let Ok(output) = std::process::Command::new("cliphist")
        .arg("list")
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout.lines().filter_map(|line| {
                let mut parts = line.splitn(2, '\t');
                let id_str = parts.next()?.trim();
                let preview = parts.next()?.trim().to_owned();
                let id = id_str.parse::<usize>().ok()?;
                Some((id, preview))
            }).collect();
        }
    }

    Vec::new()
}

pub fn copy_entry(id: usize) {
    // pipe cliphist decode <id> to wl-copy
    std::thread::spawn(move || {
        let decode_process = std::process::Command::new("cliphist")
            .arg("decode")
            .arg(id.to_string())
            .stdout(Stdio::piped())
            .spawn();

        if let Ok(mut decode_child) = decode_process {
            if let Some(mut decode_stdout) = decode_child.stdout.take() {
                let mut buffer = Vec::new();

                if let Ok(bytes_read) = decode_stdout.read_to_end(&mut buffer) {
                    if bytes_read == 0 {
                        return;
                    }

                    // if the buffer converted to lossy utf8 is one character, copy it with a output call,
                    // wl-copy doesn't like single character inputs to it's stdin for some reason
                    let lossy_utf8 = String::from_utf8_lossy(&buffer);
                    if lossy_utf8.chars().count() == 1 {
                        copy_text(&lossy_utf8);
                        return;
                    }

                    // otherwise the buffer can be piped as usual
                    let wl_copy_process = std::process::Command::new("wl-copy")
                        .stdin(Stdio::piped())
                        .spawn();

                    if let Ok(mut wl_copy_child) = wl_copy_process {
                        if let Some(mut wl_copy_stdin) = wl_copy_child.stdin.take() {
                            let _ = wl_copy_stdin.write_all(&buffer);
                        }
                        let _ = wl_copy_child.wait();
                    }
                }
            }
            let _ = decode_child.wait();
        }
    });
}

pub fn copy_text(text: &str) {
    let _ = std::process::Command::new("wl-copy")
        .arg(text)
        .output()
        .map_err(|e| println!("Failed to copy text to clipboard: {}", e));
}

pub fn spawn_indefinite_watcher(type_arg: &'static str) {
    if !process::is_command_available("wl-paste") || !process::is_command_available("cliphist") {
        println!("wl-paste or cliphist not found, cannot spawn clipboard watcher");
        return;
    }

    process::kill_task_if_any(&format!("wl-paste --type {} --watch cliphist store", type_arg));

    let text_watch = std::process::Command::new("wl-paste")
        .arg("--type")
        .arg(type_arg)
        .arg("--watch")
        .arg("cliphist")
        .arg("store")
        .spawn();

    // start this watcher again if it exits unexpectedly
    if let Ok(mut text_child) = text_watch {
        std::thread::spawn(move || {
            let _ = text_child.wait();
            spawn_indefinite_watcher(type_arg);
        });
    }
}

pub fn activate() {
    if !process::is_command_available("cliphist") || !process::is_command_available("wl-paste") {
        println!("cliphist or wl-paste not found, clipboard singleton will not be activated");
        return;
    }

    spawn_indefinite_watcher("text");
    spawn_indefinite_watcher("image");
}