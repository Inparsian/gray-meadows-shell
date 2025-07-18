use std::sync::Mutex;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{color::{is_valid_hex_color, model::Rgba}, helpers::filesystem};

const VAR_REGEX: &str = r"^\$([a-zA-Z0-9_-]+):\s*([a-zA-Z0-9#() ,.-]+);$";

pub static SCSS_VARS: Lazy<Mutex<ScssVars>> = Lazy::new(|| Mutex::new(ScssVars::new()));

pub struct ScssVars {
    string_vars: std::collections::HashMap<String, String>,
    color_vars: std::collections::HashMap<String, Rgba>,
}

impl ScssVars {
    pub fn new() -> Self {
        Self {
            string_vars: std::collections::HashMap::new(),
            color_vars: std::collections::HashMap::new(),
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
    let user_styles_path = format!("{}/user.scss", styles_dir);
    let content = std::fs::read_to_string(user_styles_path);

    if let Ok(content) = content {
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
    scss_vars.get_color(name).cloned()
}

#[allow(dead_code)]
pub fn get_string(name: &str) -> Option<String> {
    let scss_vars = SCSS_VARS.lock().unwrap();
    scss_vars.get_string(name).cloned()
}