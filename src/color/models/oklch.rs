use std::sync::LazyLock;
use regex::Regex;

pub static OKLCH_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^oklch\(\s*(\d{1,3}(?:\.\d+)?)\s+(\d{1,3}(?:\.\d+)?)\s+(\d{1,3}(?:\.\d+)?)\s*\)$").unwrap()
});

pub struct Oklch {
    pub lightness: f64,
    pub chroma: f64,
    pub hue: f64
}

impl super::ColorModel for Oklch {
    fn from_hex(hex: &str) -> Self {
        let rgba = super::Rgba::from_hex(hex);
        let linear_rgba = rgba.into_linear();

        // Linear RGB -> Cube-rooted LMS
        let lms_l = 0.051_457_565_3_f64
            .mul_add(linear_rgba.blue, 0.412_165_612_0_f64.mul_add(linear_rgba.red, 0.536_275_208_0 * linear_rgba.green))
            .cbrt();

        let lms_m = 0.107_406_579_0_f64
            .mul_add(linear_rgba.blue, 0.211_859_107_0_f64.mul_add(linear_rgba.red, 0.680_718_958_4 * linear_rgba.green))
            .cbrt();

        let lms_s = 0.629_323_455_7_f64
            .mul_add(linear_rgba.blue, 0.088_309_794_7_f64.mul_add(linear_rgba.red, 0.281_847_417_4 * linear_rgba.green))
            .cbrt();

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

    fn from_string(s: &str) -> Option<Self> {
        let captures = OKLCH_PATTERN.captures(s.trim())?;
        let lightness = captures.get(1)?.as_str().parse::<f64>().ok()?;
        let chroma = captures.get(2)?.as_str().parse::<f64>().ok()?;
        let hue = captures.get(3)?.as_str().parse::<f64>().ok()?;

        Some(Self { lightness, chroma, hue })
    }

    fn into_string(self) -> String {
        format!("oklch({:.4} {:.4} {:.2})", self.lightness, self.chroma, self.hue)
    }

    fn into_hex(self) -> String {
        // Oklch -> Oklab - L is the same.
        let hue_rad = self.hue.to_radians();
        let ok_a = self.chroma * hue_rad.cos();
        let ok_b = self.chroma * hue_rad.sin();

        // Oklab -> Cubed LMS
        let lms_l = 0.215_803_757_3_f64.mul_add(ok_b, 0.396_337_777_4_f64.mul_add(ok_a, self.lightness)).powi(3);
        let lms_m = 0.063_854_172_8_f64.mul_add(-ok_b, 0.105_561_345_8_f64.mul_add(-ok_a, self.lightness)).powi(3);
        let lms_s = 1.291_485_548_0_f64.mul_add(-ok_b, 0.089_484_177_5_f64.mul_add(-ok_a, self.lightness)).powi(3);

        // Cubed LMS -> Linear RGB
        let lrgb = super::LinearRgba {
            red: 0.230_969_929_2_f64.mul_add(lms_s, 4.076_741_662_1_f64.mul_add(lms_l, -(3.307_711_591_3 * lms_m))),
            green: 0.341_319_396_5_f64.mul_add(-lms_s, (-1.268_438_004_6_f64).mul_add(lms_l, 2.609_757_401_1 * lms_m)),
            blue: 1.707_614_701_0_f64.mul_add(lms_s, (-0.004_196_086_3_f64).mul_add(lms_l, -(0.703_418_614_7 * lms_m))),
            alpha: 1.0
        };

        // Linear RGB -> sRGBA -> Hex
        lrgb.into_rgba().into_hex()
    }
}