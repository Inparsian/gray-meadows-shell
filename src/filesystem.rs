pub fn get_project_directory() -> String {
    // Get directory relative to the closest Cargo.toml file
    let mut path = std::env::current_exe().expect("Failed to get current executable path");
    while let Some(parent) = path.parent() {
        if parent.join("Cargo.toml").exists() {
            // Found the Cargo.toml, return the project directory
            return parent.display().to_string();
        }
        path = parent.to_path_buf();
    }

    panic!("Cargo.toml not found in the path hierarchy");
}

pub fn get_styles_directory() -> String {
    let styles_dir = format!("{}/styles", get_project_directory());
    if !std::path::Path::new(&styles_dir).exists() {
        panic!("Styles directory does not exist: {}", styles_dir);
    }
    
    styles_dir
}

pub fn get_home_directory() -> String {
    std::env::var("HOME")
        .expect("HOME environment variable not set")
}

pub fn get_config_directory() -> String {
    let home = get_home_directory();
    format!("{}/.config/gray-meadows", home)
}

pub fn get_xdg_runtime_directory() -> String {
    std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| {
        let home = get_home_directory();
        format!("{}/.local/run", home)
    })
}