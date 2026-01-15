use std::sync::LazyLock;
use regex::Regex;

use crate::FLOAT_TOLERANCE;

pub static HSL_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^hsl\(\s*(\d{1,3}(?:\.\d+)?)\s*,\s*(\d{1,3}(?:\.\d+)?)%\s*,\s*(\d{1,3}(?:\.\d+)?)%\s*\)$").unwrap()
});

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsl {
    pub hue: f64,
    pub saturation: f64,
    pub lightness: f64,
}

impl super::ColorModel for Hsl {
    fn from_hex(hex: &str) -> Self {
        let rgba = super::Rgba::from_hex(hex);

        let r_normalized = rgba.red as f64 / 255.0;
        let g_normalized = rgba.green as f64 / 255.0;
        let b_normalized = rgba.blue as f64 / 255.0;

        let max = r_normalized.max(g_normalized).max(b_normalized);
        let min = r_normalized.min(g_normalized).min(b_normalized);
        let delta = max - min;

        let hue = if delta == 0.0 {
            0.0
        } else if (max - r_normalized).abs() < FLOAT_TOLERANCE {
            ((g_normalized - b_normalized) / delta + (if g_normalized < b_normalized { 6.0 } else { 0.0 })) * 60.0
        } else if (max - g_normalized).abs() < FLOAT_TOLERANCE {
            ((b_normalized - r_normalized) / delta + 2.0) * 60.0
        } else {
            ((r_normalized - g_normalized) / delta + 4.0) * 60.0
        };

        let lightness = f64::midpoint(max, min);

        let saturation = if delta > 0.0 {
            delta / if lightness <= 0.5 {
                max + min
            } else {
                2.0 - max - min
            }
        } else {
            0.0
        };

        Self {
            hue,
            saturation: (saturation * 100.0).round(),
            lightness: (lightness * 100.0).round()
        }
    }

    fn from_string(s: &str) -> Option<Self> {
        let captures = HSL_PATTERN.captures(s.trim())?;
        let hue = captures.get(1)?.as_str().parse::<f64>().ok()?;
        let saturation = captures.get(2)?.as_str().parse::<f64>().ok()?;
        let lightness = captures.get(3)?.as_str().parse::<f64>().ok()?;

        Some(Self { hue, saturation, lightness })
    }

    fn into_string(self) -> String {
        format!("hsl({:.2}, {:.2}%, {:.2}%)", self.hue, self.saturation, self.lightness)
    }

    fn into_hex(self) -> String {
        let chroma = (1.0 - 2.0_f64.mul_add(self.lightness / 100.0, -1.0).abs()) * (self.saturation / 100.0);
        let intermediate = chroma * (1.0 - ((self.hue / 60.0) % 2.0 - 1.0).abs());
        let match_value = (self.lightness / 100.0) - (chroma / 2.0);

        let normalize = |value: f64| ((value + match_value) * 255.0).round() as u8;
        let (rr, gg, bb) = if self.hue < 60.0 {
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

        format!("#{:02x}{:02x}{:02x}", normalize(rr), normalize(gg), normalize(bb))
    }
}

impl Hsl {
    pub fn h_diff(&self, b: &Self) -> f64 {
        let diff = (self.hue - b.hue).abs();
        if diff > 180.0 {
            360.0 - diff
        } else {
            diff
        }
    }

    pub fn s_diff(&self, b: &Self) -> f64 {
        (self.saturation - b.saturation).abs()
    }

    pub fn l_diff(&self, b: &Self) -> f64 {
        (self.lightness - b.lightness).abs()
    }
}