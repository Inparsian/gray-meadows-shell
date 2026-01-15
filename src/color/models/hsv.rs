use std::sync::LazyLock;
use regex::Regex;

use crate::FLOAT_TOLERANCE;

pub static HSV_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^hsv\(\s*(\d{1,3}(?:\.\d+)?)\s*,\s*(\d{1,3}(?:\.\d+)?)%\s*,\s*(\d{1,3}(?:\.\d+)?)%\s*\)$").unwrap()
});

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsv {
    pub hue: f64,
    pub saturation: f64,
    pub value: f64
}

impl super::ColorModel for Hsv {
    fn from_hex(hex: &str) -> Self {
        let rgba = super::Rgba::from_hex(hex);

        let max = rgba.red.max(rgba.green).max(rgba.blue) as f64;
        let min = rgba.red.min(rgba.green).min(rgba.blue) as f64;
        let delta = max - min;

        let hue = if delta == 0.0 {
            0.0
        } else if (max - rgba.red as f64).abs() < FLOAT_TOLERANCE {
            ((rgba.green as f64 - rgba.blue as f64) / delta + (if rgba.green < rgba.blue { 6.0 } else { 0.0 })) * 60.0
        } else if (max - rgba.green as f64).abs() < FLOAT_TOLERANCE {
            ((rgba.blue as f64 - rgba.red as f64) / delta + 2.0) * 60.0
        } else {
            ((rgba.red as f64 - rgba.green as f64) / delta + 4.0) * 60.0
        };

        let saturation = if max == 0.0 {
            0.0
        } else {
            (delta / max) * 100.0
        };

        let value = (max / 255.0) * 100.0;

        Self { hue, saturation, value }
    }

    fn from_string(s: &str) -> Option<Self> {
        let captures = HSV_PATTERN.captures(s.trim())?;
        let hue = captures.get(1)?.as_str().parse::<f64>().ok()?;
        let saturation = captures.get(2)?.as_str().parse::<f64>().ok()?;
        let value = captures.get(3)?.as_str().parse::<f64>().ok()?;

        Some(Self { hue, saturation, value })
    }

    fn into_hex(self) -> String {
        let chroma = (self.value / 100.0) * (self.saturation / 100.0);
        let intermediate = chroma * (1.0 - ((self.hue / 60.0) % 2.0 - 1.0).abs());
        let match_value = (self.value / 100.0) - chroma;

        let (red_prime, green_prime, blue_prime) = if self.hue < 60.0 {
            (chroma, intermediate, 0.0)
        } else if self.hue < 120.0 {
            (intermediate, chroma, 0.0)
        } else if self.hue < 180.0 {
            (0.0, chroma, intermediate)
        } else if self.hue < 240.0 {
            (0.0, intermediate, chroma)
        } else if self.hue < 300.0 {
            (intermediate, 0.0, chroma)
        } else {
            (chroma, 0.0, intermediate)
        };

        let rr = (red_prime + match_value) * 255.0;
        let gg = (green_prime + match_value) * 255.0;
        let bb = (blue_prime + match_value) * 255.0;

        format!(
            "#{:02x}{:02x}{:02x}",
            rr.round() as u8,
            gg.round() as u8,
            bb.round() as u8
        )
    }
    
    fn into_string(self) -> String {
        format!("hsv({:.2}, {:.2}%, {:.2}%)", self.hue, self.saturation, self.value)
    }
}