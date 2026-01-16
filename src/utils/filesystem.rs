pub fn get_home_directory() -> String {
    std::env::var("HOME")
        .expect("HOME environment variable not set")
}

pub fn get_config_directory() -> String {
    let home = get_home_directory();
    format!("{}/.config/gray-meadows", home)
}

pub fn get_styles_directory() -> String {
    format!("{}/styles", get_config_directory())
}

pub fn get_local_data_directory() -> String {
    format!("{}/.local/share/gray-meadows", get_home_directory())
}

pub fn get_local_state_directory() -> String {
    format!("{}/.local/state/gray-meadows", get_home_directory())
}

pub fn get_xdg_runtime_directory() -> String {
    std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| {
        format!("{}/.local/run", get_home_directory())
    })
}