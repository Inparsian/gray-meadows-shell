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
    pub fn as_rgba(&self) -> Rgba {
        let hex = self.as_hex();
        Rgba::from_hex(&hex)
    }

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
}