use std::sync::LazyLock;
use regex::Regex;

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

    fn into_hex(self) -> String {
        if self.alpha < 1 {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.red, self.green, self.blue, self.alpha)
        } else {
            format!("#{:02x}{:02x}{:02x}", self.red, self.green, self.blue)
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
    pub fn into_linear(self) -> LinearRgba {
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
}

/// A linear RGBA color model where the RGB components are in linear space.
pub struct LinearRgba {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
    pub alpha: f64
}

impl LinearRgba {
    pub fn into_rgba(self) -> Rgba {
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