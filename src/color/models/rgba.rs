use std::sync::LazyLock;
use regex::Regex;

const HUE_SECTOR_COUNT: f64 = 6.0;
const DEGREES_PER_HUE_SECTOR: f64 = 60.0;
const GREEN_HUE_SECTOR_OFFSET: f64 = 2.0;
const BLUE_HUE_SECTOR_OFFSET: f64 = 4.0;

pub static RGB_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^rgb\(\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*(\d{1,3})\s*\)$").unwrap()
});

pub static RGBA_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^rgba\(\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*(0|0?\.\d+|1(\.0)?)\s*\)$").unwrap()
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl super::ColorModel for Rgba {
    // rgba is rgba, duh doy, the floor is also made of floor
    fn from_rgba(rgba: Rgba) -> Self {
        rgba
    }

    fn into_rgba(self) -> Rgba {
        self
    }
    
    // hex bidirectional conversions should be a Rgba responsibility
    /// Directly gets this color from a hex string.
    fn from_hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        let mut components: Vec<u8> = match hex.len() {
            3 | 4 => hex.chars()
                .map(|c| u8::from_str_radix(&c.to_string().repeat(2), 16).unwrap_or_default())
                .collect(),

            6 | 8 => (0..hex.len()).step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or_default())
                .collect(),

            _ => vec![0, 0, 0, 255], // Default to black with full opacity
        };

        // Default alpha to 255 if not provided
        if components.len() == 3 {
            components.push(255);
        }

        Self {
            red: components[0],
            green: components[1],
            blue: components[2],
            alpha: components[3]
        }
    }

    /// Directly converts this color into a hex string.
    fn into_hex(self) -> String {
        if self.alpha < 1 {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.red, self.green, self.blue, self.alpha)
        } else {
            format!("#{:02x}{:02x}{:02x}", self.red, self.green, self.blue)
        }
    }
    
    fn from_string(s: &str) -> Option<Self> {
        if let Some(captures) = RGB_PATTERN.captures(s.trim()) {
            let red = captures.get(1)?.as_str().parse::<u8>().ok()?;
            let green = captures.get(2)?.as_str().parse::<u8>().ok()?;
            let blue = captures.get(3)?.as_str().parse::<u8>().ok()?;

            Some(Self { red, green, blue, alpha: 255 })
        } else if let Some(captures) = RGBA_PATTERN.captures(s.trim()) {
            let red = captures.get(1)?.as_str().parse::<u8>().ok()?;
            let green = captures.get(2)?.as_str().parse::<u8>().ok()?;
            let blue = captures.get(3)?.as_str().parse::<u8>().ok()?;
            let alpha_float = captures.get(4)?.as_str().parse::<f64>().ok()?;
            let alpha = (alpha_float * 255.0).round() as u8;

            Some(Self { red, green, blue, alpha })
        } else {
            None
        }
    }
    
    fn into_string(self) -> String {
        if self.alpha < 255 {
            format!("rgba({}, {}, {}, {:.2})", self.red, self.green, self.blue, self.alpha as f64 / 255.0)
        } else {
            format!("rgb({}, {}, {})", self.red, self.green, self.blue)
        }
    }
}

impl Rgba {
    pub(super) fn from_srgb(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red: red.clamp(0, 255),
            green: green.clamp(0, 255),
            blue: blue.clamp(0, 255),
            alpha: 255
        }
    }
    
    pub(super) fn into_linear(self) -> LinearRgba {
        fn gamma_to_linear(value: f64) -> f64 {
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        }

        LinearRgba {
            red: gamma_to_linear(self.red as f64 / 255.0),
            green: gamma_to_linear(self.green as f64 / 255.0),
            blue: gamma_to_linear(self.blue as f64 / 255.0),
            alpha: self.alpha as f64 / 255.0
        }
    }
    
    pub(super) fn into_normalized(self) -> NormalizedRgba {
        NormalizedRgba {
            red: self.red as f64 / 255.0,
            green: self.green as f64 / 255.0,
            blue: self.blue as f64 / 255.0,
            alpha: self.alpha as f64 / 255.0,
        }
    }
}

/// A linear RGBA color model where the RGB components are in linear space.
pub(super) struct LinearRgba {
    pub(super) red: f64,
    pub(super) green: f64,
    pub(super) blue: f64,
    pub(super) alpha: f64,
}

impl LinearRgba {
    pub(super) fn into_rgba(self) -> Rgba {
        fn linear_to_gamma(value: f64) -> u8 {
            if value <= 0.003_130_8 {
                (value * 12.92 * 255.0).round() as u8
            } else {
                1.055_f64.mul_add(value.powf(1.0 / 2.4), -0.055).mul_add(255.0, 0.0).round() as u8
            }
        }

        Rgba {
            red: linear_to_gamma(self.red),
            green: linear_to_gamma(self.green),
            blue: linear_to_gamma(self.blue),
            alpha: (self.alpha * 255.0).round() as u8
        }
    }
}

/// A normalized RGBA color model where the RGB components are in the range [0.0, 1.0].
pub(super) struct NormalizedRgba {
    pub(super) red: f64,
    pub(super) green: f64,
    pub(super) blue: f64,
    pub(super) alpha: f64,
}

impl NormalizedRgba {
    pub(super) fn from_nsrgb(red: f64, green: f64, blue: f64) -> Self {
        Self { red, green, blue, alpha: 1.0 }
    }
    
    pub(super) fn into_rgba(self) -> Rgba {
        fn denormalize(value: f64) -> u8 {
            (value.clamp(0.0, 1.0) * 255.0).round() as u8
        }
        
        Rgba {
            red: denormalize(self.red),
            green: denormalize(self.green),
            blue: denormalize(self.blue),
            alpha: denormalize(self.alpha),
        }
    }
    
    pub(super) fn max_rgb(&self) -> f64 {
        self.red.max(self.green).max(self.blue)
    }

    pub(super) fn min_rgb(&self) -> f64 {
        self.red.min(self.green).min(self.blue)
    }

    /// Calculate this color's hue in degrees for the HSL and HSV color models.
    ///
    /// The caller must ensure that `delta > 0.0` (i.e. the color is chromatic).
    /// `delta` is passed to avoid redundant calculations.
    pub(super) fn hue_degrees(&self, delta: f64) -> f64 {
        debug_assert!(delta > 0.0, "delta must be greater than 0.0 for hue calculation");
        let hue_sector = if self.red >= self.green && self.red >= self.blue {
            let red_hue_sector_offset = if self.green < self.blue { HUE_SECTOR_COUNT } else { 0.0 };
            (self.green - self.blue) / delta + red_hue_sector_offset
        } else if self.green >= self.blue {
            (self.blue - self.red) / delta + GREEN_HUE_SECTOR_OFFSET
        } else {
            (self.red - self.green) / delta + BLUE_HUE_SECTOR_OFFSET
        };
        hue_sector * DEGREES_PER_HUE_SECTOR
    }
}