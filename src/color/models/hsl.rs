use std::sync::LazyLock;
use regex::Regex;

use super::{Rgba, rgba::NormalizedRgba};

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
    fn from_rgba(rgba: Rgba) -> Self {
        let nrgba = rgba.into_normalized();
        let (max, min) = (nrgba.max_rgb(), nrgba.min_rgb());
        let (sum, delta) = (max + min, max - min);
        let lightness = f64::midpoint(max, min);
        let hue = if delta > 0.0 { nrgba.hue_degrees(delta) } else { 0.0 };
        let saturation = if delta > 0.0 { delta / if lightness <= 0.5 { sum } else { 2.0 - sum } } else { 0.0 };

        Self {
            hue,
            saturation: (saturation * 100.0).round(),
            lightness: (lightness * 100.0).round(),
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

    fn into_rgba(self) -> Rgba {
        // this alternative formula simplifies the polygonal piecewise functions using
        // min & max values as well as a channel function
        // see https://en.wikipedia.org/wiki/HSL_and_HSV#HSL_to_RGB_alternative
        let (lightness_ratio, saturation_ratio) = (self.lightness / 100.0, self.saturation / 100.0);
        let h = self.hue.rem_euclid(360.0) / 30.0;
        let a = saturation_ratio * lightness_ratio.min(1.0 - lightness_ratio);
        let f = |n: f64| -> f64 {
            let k = (n + h).rem_euclid(12.0);
            lightness_ratio - a * (-1.0_f64).max((k - 3.0).min((9.0 - k).min(1.0)))
        };
        NormalizedRgba::from_nsrgb(f(0.0), f(8.0), f(4.0)).into_rgba()
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