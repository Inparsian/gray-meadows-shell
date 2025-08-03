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

    pub fn as_linear(self) -> LinearRgba {
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

pub struct LinearRgba {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
    pub alpha: f64
}

impl LinearRgba {
    pub fn as_rgba(&self) -> Rgba {
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsv {
    pub hue: f64,
    pub saturation: f64,
    pub value: f64
}

impl Hsv {
    pub fn from_hex(hex: &str) -> Self {
        let rgba = Rgba::from_hex(hex);

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

    pub fn as_oklch(&self) -> Oklch {
        let hex = self.as_hex();
        Oklch::from_hex(&hex)
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

pub struct Oklch {
    pub lightness: f64,
    pub chroma: f64,
    pub hue: f64
}

impl Oklch {
    pub fn from_hex(hex: &str) -> Self {
        let rgba = Rgba::from_hex(hex);
        let linear_rgba = rgba.as_linear();

        // Linear RGB -> Cube-rooted LMS
        let lms_l = 0.051_457_565_3_f64.mul_add(linear_rgba.blue, 0.412_165_612_0_f64.mul_add(linear_rgba.red, 0.536_275_208_0 * linear_rgba.green)).cbrt();
        let lms_m = 0.107_406_579_0_f64.mul_add(linear_rgba.blue, 0.211_859_107_0_f64.mul_add(linear_rgba.red, 0.680_718_958_4 * linear_rgba.green)).cbrt();
        let lms_s = 0.629_323_455_7_f64.mul_add(linear_rgba.blue, 0.088_309_794_7_f64.mul_add(linear_rgba.red, 0.281_847_417_4 * linear_rgba.green)).cbrt();

        // LMS -> Oklab
        let lightness = 0.004_072_046_8_f64.mul_add(-lms_s, 0.210_454_255_3_f64.mul_add(lms_l, 0.793_617_785_0 * lms_m));
        let ok_a = 0.450_593_709_9_f64.mul_add(lms_s, 1.977_998_495_1_f64.mul_add(lms_l, -(2.428_592_205_0 * lms_m)));
        let ok_b = 0.808_675_766_0_f64.mul_add(-lms_s, 0.025_904_037_1_f64.mul_add(lms_l, 0.782_771_766_2 * lms_m));

        // Oklab -> Oklch
        let chroma = ok_a.hypot(ok_b);
        let hue = if chroma == 0.0 {
            0.0
        } else {
            (ok_b.atan2(ok_a).to_degrees() + 360.0) % 360.0
        };

        Self { lightness, chroma, hue }
    }

    pub fn as_hex(&self) -> String {
        // Oklch -> Oklab - L is the same.
        let hue_rad = self.hue.to_radians();
        let ok_a = self.chroma * hue_rad.cos();
        let ok_b = self.chroma * hue_rad.sin();

        // Oklab -> Cubed LMS
        let lms_l = 0.215_803_757_3_f64.mul_add(ok_b, 0.396_337_777_4_f64.mul_add(ok_a, self.lightness)).powi(3);
        let lms_m = 0.063_854_172_8_f64.mul_add(-ok_b, 0.105_561_345_8_f64.mul_add(-ok_a, self.lightness)).powi(3);
        let lms_s = 1.291_485_548_0_f64.mul_add(-ok_b, 0.089_484_177_5_f64.mul_add(-ok_a, self.lightness)).powi(3);

        // Cubed LMS -> Linear RGB
        let lrgb = LinearRgba {
            red: 0.230_969_929_2_f64.mul_add(lms_s, 4.076_741_662_1_f64.mul_add(lms_l, -(3.307_711_591_3 * lms_m))),
            green: 0.341_319_396_5_f64.mul_add(-lms_s, (-1.268_438_004_6_f64).mul_add(lms_l, 2.609_757_401_1 * lms_m)),
            blue: 1.707_614_701_0_f64.mul_add(lms_s, (-0.004_196_086_3_f64).mul_add(lms_l, -(0.703_418_614_7 * lms_m))),
            alpha: 1.0
        };

        // Linear RGB -> sRGBA -> Hex
        lrgb.as_rgba().as_hex()
    }
}

pub fn int_to_hex(int: u32) -> String {
    format!("#{:06x}", int)
}