#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
    pub alpha: f64,
}

impl Rgba {
    pub fn from_hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        let len = hex.len();
        let (r, g, b, a) = if len == 6 || len == 8 {
            (
                u8::from_str_radix(&hex[0..2], 16).unwrap(),
                u8::from_str_radix(&hex[2..4], 16).unwrap(),
                u8::from_str_radix(&hex[4..6], 16).unwrap(),
                if len == 8 {
                    u8::from_str_radix(&hex[6..8], 16).unwrap()
                } else {
                    255
                },
            )
        } else if len == 3 || len == 4 {
            (
                u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap(),
                u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap(),
                u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap(),
                if len == 4 {
                    u8::from_str_radix(&hex[3..4].repeat(2), 16).unwrap()
                } else {
                    255
                },
            )
        } else {
            return Self { red: 0.0, green: 0.0, blue: 0.0, alpha: 1.0 };
        };

        Self {
            red: r as f64 / 255.0,
            green: g as f64 / 255.0,
            blue: b as f64 / 255.0,
            alpha: a as f64 / 255.0,
        }
    }
}

pub fn is_valid_hex_color(hex: &str) -> bool {
    let hex = hex.trim_start_matches('#');
    let len = hex.len();
    (len == 6 || len == 8 || len == 3 || len == 4) && hex.chars().all(|c| c.is_ascii_hexdigit())
}