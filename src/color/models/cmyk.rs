use std::sync::LazyLock;
use regex::Regex;

use super::Rgba;

pub static CMYK_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^cmyk\(\s*(\d{1,3})%\s*,\s*(\d{1,3})%\s*,\s*(\d{1,3})%\s*,\s*(\d{1,3})%\s*\)$").unwrap()
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cmyk {
    pub cyan: u8,
    pub magenta: u8,
    pub yellow: u8,
    pub black: u8,
}

impl super::ColorModel for Cmyk {
    fn from_rgba(rgba: Rgba) -> Self {
        let nrgba = rgba.into_normalized();
        let black = 1.0 - nrgba.max_rgb();
        let convert = |component: f64| -> u8 {
            (((1.0 - component - black) / (1.0 - black)) * 100.0).round() as u8
        };

        Self {
            cyan: convert(nrgba.red),
            magenta: convert(nrgba.green),
            yellow: convert(nrgba.blue),
            black: (black * 100.0).round() as u8
        }
    }

    fn from_string(s: &str) -> Option<Self> {
        let captures = CMYK_PATTERN.captures(s.trim())?;
        let cyan = captures.get(1)?.as_str().parse::<u8>().ok()?;
        let magenta = captures.get(2)?.as_str().parse::<u8>().ok()?;
        let yellow = captures.get(3)?.as_str().parse::<u8>().ok()?;
        let black = captures.get(4)?.as_str().parse::<u8>().ok()?;

        Some(Self { cyan, magenta, yellow, black })
    }

    fn into_string(self) -> String {
        format!("cmyk({}%, {}%, {}%, {}%)", self.cyan, self.magenta, self.yellow, self.black)
    }

    fn into_rgba(self) -> Rgba {
        let k_ratio = self.black as f64 / 100.0;
        let convert = |component: u8| -> u8 {
            ((1.0 - (component as f64 / 100.0)).mul_add(1.0 - k_ratio, k_ratio) * 255.0).round() as u8
        };

        Rgba::from_srgb(convert(self.cyan), convert(self.magenta), convert(self.yellow))
    }
}