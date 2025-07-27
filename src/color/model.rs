const FLOAT_TOLERANCE: f64 = 0.0001;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Rgba {
    pub fn from_hex(hex: &str) -> Self {
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

    pub fn as_hex(self) -> String {
        if self.alpha < 1 {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.red, self.green, self.blue, self.alpha)
        } else {
            format!("#{:02x}{:02x}{:02x}", self.red, self.green, self.blue)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsv {
    pub hue: f64,
    pub saturation: f64,
    pub value: f64
}

impl Hsv {
    pub fn as_hex(&self) -> String {
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

    pub fn as_int(self) -> u32 {
        // hex -> int
        u32::from_str_radix(self.as_hex().trim_start_matches('#'), 16).unwrap_or(0)
    }

    pub fn as_rgba(&self) -> Rgba {
        let hex = self.as_hex();
        Rgba::from_hex(&hex)
    }

    pub fn as_hsl(&self) -> Hsl {
        let hex = self.as_hex();
        Hsl::from_hex(&hex)
    }

    pub fn as_cmyk(&self) -> Cmyk {
        let hex = self.as_hex();
        Cmyk::from_hex(&hex)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsl {
    pub hue: f64,
    pub saturation: f64,
    pub lightness: f64
}

impl Hsl {
    pub fn from_hex(hex: &str) -> Self {
        let rgba = Rgba::from_hex(hex);

        let max = rgba.red.max(rgba.green).max(rgba.blue) as f64;
        let min = rgba.red.min(rgba.green).min(rgba.blue) as f64;
        let delta = max - min;

        let lightness = f64::midpoint(max, min) * 100.0;

        let mut hue = if delta < FLOAT_TOLERANCE {
            0.0
        } else if (max - rgba.red as f64).abs() < FLOAT_TOLERANCE {
            ((rgba.green as f64 - rgba.blue as f64) / delta) % 6.0 * 60.0
        } else if (max - rgba.green as f64).abs() < FLOAT_TOLERANCE {
            ((rgba.blue as f64 - rgba.red as f64) / delta + 2.0) * 60.0
        } else {
            ((rgba.red as f64 - rgba.green as f64) / delta + 4.0) * 60.0
        };

        if hue < 0.0 {
            hue += 360.0;
        }

        let saturation = if (max - min).abs() < FLOAT_TOLERANCE { 
            0.0 
        } else if lightness < 50.0 { 
            (delta / (max + min)) * 100.0 
        } else { 
            (delta / (2.0 - max - min)) * 100.0 
        };

        Self {
            hue,
            saturation,
            lightness
        }
    }

    #[allow(dead_code)] // TODO: Remove when HSL -> HSV conversion is implemented in the GUI
    pub fn as_hex(&self) -> String {
        let chroma = (self.lightness / 100.0).abs().mul_add(-2.0, 1.0) * (self.saturation / 100.0);
        let intermediate = chroma * (1.0 - ((self.hue / 60.0) % 2.0 - 1.0).abs());
        let match_value = (self.lightness / 100.0) - chroma / 2.0;

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
}

pub struct Cmyk {
    pub cyan: u8,
    pub magenta: u8,
    pub yellow: u8,
    pub black: u8
}

impl Cmyk {
    pub fn from_hex(hex: &str) -> Self {
        let rgba = Rgba::from_hex(hex);
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

    #[allow(dead_code)] // TODO: Remove when CMYK -> HSV conversion is implemented in the GUI
    pub fn as_hex(&self) -> String {
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