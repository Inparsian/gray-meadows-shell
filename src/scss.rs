use std::{collections::HashMap, path::Path};
use std::sync::{LazyLock, Mutex};
use notify::{EventKind, Watcher as _};
use notify::event::{AccessKind, AccessMode};
use regex::Regex;

use crate::APP_LOCAL;
use crate::utils::filesystem;
use crate::color::{is_valid_hex_color, model::Rgba};

const VAR_REGEX: &str = r#"^\$([a-zA-Z0-9_-]+):\s*(?:"?([^"]+)"?|([a-zA-Z0-9#() ,.-])+);$"#;

pub static SCSS_VARS: LazyLock<Mutex<ScssVars>> = LazyLock::new(|| Mutex::new(ScssVars::new()));

pub struct ScssVars {
    string_vars: HashMap<String, String>,
    color_vars: HashMap<String, Rgba>,
}

impl ScssVars {
    pub fn new() -> Self {
        Self {
            string_vars: HashMap::new(),
            color_vars: HashMap::new(),
        }
    }

    pub fn add_string(&mut self, name: String, value: String) {
        self.string_vars.insert(name, value);
    }

    pub fn add_color(&mut self, name: String, value: Rgba) {
        self.color_vars.insert(name, value);
    }

    #[allow(dead_code)]
    pub fn get_string(&self, name: &str) -> Option<&String> {
        self.string_vars.get(name)
    }

    pub fn get_color(&self, name: &str) -> Option<&Rgba> {
        self.color_vars.get(name)
    }
}

pub fn refresh_variables() {
    let mut vars = ScssVars::new();
    let regex = Regex::new(VAR_REGEX).unwrap();

    let styles_dir = filesystem::get_styles_directory();
    let user_styles_path = format!("{}/_user.scss", styles_dir);

    if let Ok(content) = std::fs::read_to_string(user_styles_path) {
        for line in content.lines() {
            if let Some(caps) = regex.captures(line) {
                let name = caps[1].to_string();
                let value = caps[2].to_string();

                if is_valid_hex_color(&value) {
                    vars.add_color(name, Rgba::from_hex(&value));
                } else {
                    vars.add_string(name, value);
                }
            }
        }
    }

    let mut scss_vars = SCSS_VARS.lock().unwrap();
    *scss_vars = vars;
}

pub fn escape_html(input: char) -> String {
    match input {
        '&' => "&amp;".to_owned(),
        '<' => "&lt;".to_owned(),
        '>' => "&gt;".to_owned(),
        '"' => "&quot;".to_owned(),
        '\'' => "&#39;".to_owned(),
        _ => input.to_string(),
    }
}

pub fn get_color(name: &str) -> Option<Rgba> {
    let scss_vars = SCSS_VARS.lock().unwrap();
    scss_vars.get_color(name).copied()
}

#[allow(dead_code)]
pub fn get_string(name: &str) -> Option<String> {
    let scss_vars = SCSS_VARS.lock().unwrap();
    scss_vars.get_string(name).cloned()
}

pub fn bundle_apply_scss() {
    gtk4::glib::MainContext::default().invoke(|| {
        let styles_path = filesystem::get_styles_directory();
        
        // Run sass
        let output = std::process::Command::new("sass")
            .arg(format!("-I {}", styles_path))
            .arg(format!("{}/output.css", styles_path))
            .output()
            .expect("Failed to run sass command");
        
        if !output.status.success() {
            error!(stderr = %String::from_utf8_lossy(&output.stderr), "Error running sass");
            return;
        }
    
        // Load the generated CSS into the provider
        let css = std::fs::read_to_string(format!("{}/output.css", styles_path))
            .expect("Failed to read output.css");

        APP_LOCAL.with(|app| app.provider.load_from_data(&css));
        
        refresh_variables();
    });
}

pub fn watch_scss() {
    tokio::spawn(async move {
        let (tx, rx) = std::sync::mpsc::channel();
        let styles_path = filesystem::get_styles_directory();

        let mut watcher = notify::recommended_watcher(tx).unwrap();
        let result = watcher.watch(
            Path::new(&styles_path),
            notify::RecursiveMode::Recursive,
        );

        if result.is_ok() {
            info!(%styles_path, "Watching styles directory");

            for res in rx {
                match res {
                    // If the event kind is Access(Close(Write)), it means the file is done being written to
                    Ok(event) => if event.paths.iter().any(|p| p.extension() == Some("scss".as_ref())
                        && matches!(event.kind, EventKind::Access(AccessKind::Close(AccessMode::Write))))
                    {
                        debug!(paths = ?event.paths, "Styles changed");
                        bundle_apply_scss();
                    },

                    Err(e) => {
                        error!(%e, "Error watching styles directory");
                    }
                }
            }
        } else {
            error!(error = %result.unwrap_err(), "Failed to watch styles directory");
        }
    });
}