use std::sync::LazyLock;
use regex::Regex;

pub static OKLCH_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^oklch\(\s*(\d{1,3}(?:\.\d+)?)\s+(\d{1,3}(?:\.\d+)?)\s+(\d{1,3}(?:\.\d+)?)\s*\)$").unwrap()
});

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oklch {
    pub lightness: f64,
    pub chroma: f64,
    pub hue: f64,
}

impl super::ColorModel for Oklch {
    fn from_hex(hex: &str) -> Self {
        let oklab = super::Oklab::from_hex(hex);
        let chroma = oklab.a.hypot(oklab.b);
        let hue = if chroma == 0.0 {
            0.0
        } else {
            (oklab.b.atan2(oklab.a).to_degrees() + 360.0) % 360.0
        };

        Self {
            lightness: oklab.lightness,
            chroma,
            hue,
        }
    }

    fn from_string(s: &str) -> Option<Self> {
        let captures = OKLCH_PATTERN.captures(s.trim())?;
        let lightness = captures.get(1)?.as_str().parse::<f64>().ok()?;
        let chroma = captures.get(2)?.as_str().parse::<f64>().ok()?;
        let hue = captures.get(3)?.as_str().parse::<f64>().ok()?;

        Some(Self { lightness, chroma, hue })
    }

    fn into_string(self) -> String {
        format!("oklch({:.3} {:.3} {:.2})", self.lightness, self.chroma, self.hue)
    }

    fn into_hex(self) -> String {
        let hue_rad = self.hue.to_radians();
        super::Oklab {
            lightness: self.lightness,
            a: self.chroma * hue_rad.cos(),
            b: self.chroma * hue_rad.sin(),
        }.into_hex()
    }
}