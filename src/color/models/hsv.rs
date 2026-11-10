use std::sync::LazyLock;
use regex::Regex;

use super::{Rgba, rgba::NormalizedRgba};

pub static HSV_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^hsv\(\s*(\d{1,3}(?:\.\d+)?)\s*,\s*(\d{1,3}(?:\.\d+)?)%\s*,\s*(\d{1,3}(?:\.\d+)?)%\s*\)$").unwrap()
});

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsv {
    pub hue: f64,
    pub saturation: f64,
    pub value: f64,
}

impl super::ColorModel for Hsv {
    fn from_rgba(rgba: Rgba) -> Self {
        let nrgba = rgba.into_normalized();
        let (max, min) = (nrgba.max_rgb(), nrgba.min_rgb());
        let delta = max - min;
        let hue = if delta > 0.0 { nrgba.hue_degrees(delta) } else { 0.0 };
        let saturation = if max > 0.0 { (delta / max) * 100.0 } else { 0.0 };

        Self { hue, saturation, value: max * 100.0 }
    }

    fn from_string(s: &str) -> Option<Self> {
        let captures = HSV_PATTERN.captures(s.trim())?;
        let hue = captures.get(1)?.as_str().parse::<f64>().ok()?;
        let saturation = captures.get(2)?.as_str().parse::<f64>().ok()?;
        let value = captures.get(3)?.as_str().parse::<f64>().ok()?;

        Some(Self { hue, saturation, value })
    }

    fn into_rgba(self) -> Rgba {
        // this alternative formula simplifies the polygonal piecewise functions using
        // min & max values as well as a channel function
        // see https://en.wikipedia.org/wiki/HSL_and_HSV#HSV_to_RGB_alternative
        let (value_ratio, saturation_ratio) = (self.value / 100.0, self.saturation / 100.0);
        let h = self.hue.rem_euclid(360.0) / 60.0;
        let f = |n: f64| -> f64 {
            let k = (n + h).rem_euclid(6.0);
            let factor = -0.0_f64.max(k.min(4.0 - k).min(1.0));
            (value_ratio * saturation_ratio).mul_add(factor, value_ratio)
        };
        NormalizedRgba::from_nsrgb(f(5.0), f(3.0), f(1.0)).into_rgba()
    }
    
    fn into_string(self) -> String {
        format!("hsv({:.2}, {:.2}%, {:.2}%)", self.hue, self.saturation, self.value)
    }
}