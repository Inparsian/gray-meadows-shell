use std::sync::LazyLock;
use regex::Regex;

pub static CMYK_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^cmyk\(\s*(\d{1,3})%\s*,\s*(\d{1,3})%\s*,\s*(\d{1,3})%\s*,\s*(\d{1,3})%\s*\)$").unwrap()
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cmyk {
    pub cyan: u8,
    pub magenta: u8,
    pub yellow: u8,
    pub black: u8
}

impl super::ColorModel for Cmyk {
    fn from_hex(hex: &str) -> Self {
        let rgba = super::Rgba::from_hex(hex);
        let r_normalized = rgba.red as f64 / 255.0;
        let g_normalized = rgba.green as f64 / 255.0;
        let b_normalized = rgba.blue as f64 / 255.0;

        let black = 1.0 - r_normalized.max(g_normalized).max(b_normalized);
        let cyan = (1.0 - r_normalized - black) / (1.0 - black);
        let magenta = (1.0 - g_normalized - black) / (1.0 - black);
        let yellow = (1.0 - b_normalized - black) / (1.0 - black);

        Self {
            cyan: (cyan * 100.0).round() as u8,
            magenta: (magenta * 100.0).round() as u8,
            yellow: (yellow * 100.0).round() as u8,
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

    fn into_hex(self) -> String {
        let cdiv = self.cyan as f64 / 100.0;
        let mdiv = self.magenta as f64 / 100.0;
        let ydiv = self.yellow as f64 / 100.0;
        let kdiv = self.black as f64 / 100.0;

        let r = (1.0 - cdiv.mul_add(1.0 - kdiv, kdiv)) * 255.0;
        let g = (1.0 - mdiv.mul_add(1.0 - kdiv, kdiv)) * 255.0;
        let b = (1.0 - ydiv.mul_add(1.0 - kdiv, kdiv)) * 255.0;

        format!("#{:02x}{:02x}{:02x}", r.round() as u8, g.round() as u8, b.round() as u8)
    }
}