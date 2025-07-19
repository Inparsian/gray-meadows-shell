pub mod model;

pub fn is_valid_hex_color(hex: &str) -> bool {
    let hex = hex.trim_start_matches('#');
    let len = hex.len();

    (len == 6 || len == 8 || len == 3 || len == 4) && hex.chars().all(|c| c.is_ascii_hexdigit())
}