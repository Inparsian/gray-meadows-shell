use self::model::{Hsv, int_to_hex};

const FLOAT_TOLERANCE: f64 = 0.0001;

pub mod model;

pub struct LighterDarkerResult {
    pub hsv: Hsv,
    pub lightness: f64,
    pub is_original: bool
}

pub fn is_valid_hex_color(hex: &str) -> bool {
    let hex = hex.trim_start_matches('#');
    let len = hex.len();

    (len == 6 || len == 8 || len == 3 || len == 4) && hex.chars().all(|c| c.is_ascii_hexdigit())
}

pub fn is_valid_int_color(int_str: &str) -> bool {
    model::INT_PATTERN.is_match(int_str.trim())
}

pub fn parse_color_into_hex(string: &str) -> Option<String> {
    if string.starts_with('#') && is_valid_hex_color(string) {
        return Some(string.to_owned());
    }

    if is_valid_int_color(string) {
        if let Ok(int_value) = string.trim().parse::<u32>() {
            return Some(int_to_hex(int_value));
        }
    }

    if let Some(rgba) = model::Rgba::from_string(string) {
        return Some(rgba.as_hex());
    }

    if let Some(hsv) = model::Hsv::from_string(string) {
        return Some(hsv.as_hex());
    }

    if let Some(hsl) = model::Hsl::from_string(string) {
        return Some(hsl.as_hex());
    }

    if let Some(cmyk) = model::Cmyk::from_string(string) {
        return Some(cmyk.as_hex());
    }

    if let Some(oklch) = model::Oklch::from_string(string) {
        return Some(oklch.as_hex());
    }

    None
}

pub fn get_analogous_colors(hsv: Hsv, count: u32) -> Vec<Hsv> {
    let mut colors = Vec::new();
    let step = 360.0 / count as f64;

    for i in 0..count {
        let mut new_hsv = hsv;
        new_hsv.hue = (i as f64).mul_add(step, hsv.hue) % 360.0;
        colors.push(new_hsv);
    }

    colors
}

pub fn get_lighter_darker_colors(base_hsv: Hsv, count: u32) -> Vec<LighterDarkerResult> {
    let mut colors: Vec<model::Hsl> = vec![base_hsv.as_hsl()];

    for i in 0..=count {
        let mut lighter_hsl = base_hsv.as_hsl();
        lighter_hsl.lightness = (i as f64).mul_add(-(100.0 / count as f64), 100.0);
        colors.push(lighter_hsl);
    }

    // Sort by lightness descending
    colors.sort_by(|a, b| b.lightness.partial_cmp(&a.lightness).unwrap_or(std::cmp::Ordering::Equal));

    // Remove duplicates
    colors.dedup_by(|a, b| {
        (a.hue - b.hue).abs() < FLOAT_TOLERANCE && 
        (a.saturation - b.saturation).abs() < FLOAT_TOLERANCE && 
        (a.lightness - b.lightness).abs() < FLOAT_TOLERANCE
    });

    // Whatever color is closest to the original color, mark it as the original
    let original_color = colors.iter().min_by(|a, b| {
        let a_diff = (a.lightness - base_hsv.as_hsl().lightness).abs();
        let b_diff = (b.lightness - base_hsv.as_hsl().lightness).abs();
        a_diff.partial_cmp(&b_diff).unwrap_or(std::cmp::Ordering::Equal)
    }).copied();

    // Remove the color that is closest to the original from the list
    // to ensure the count is correct. If the original lightness is divisible
    // by exactly 5, it'll have been deduped already at this point.
    if let Some(original) = original_color {
        if (original.lightness % 5.0) >= FLOAT_TOLERANCE {
            let _ = colors.iter().enumerate()
                .filter(|(_, c)| (c.lightness % 5.0).abs() <= FLOAT_TOLERANCE)
                .min_by(|(_, a), (_, b)| {
                    let a_diff = (a.lightness - original.lightness).abs();
                    let b_diff = (b.lightness - original.lightness).abs();
                    a_diff.partial_cmp(&b_diff).unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(i, _)| i)
                .is_some_and(|i| {
                    colors.remove(i);
                    true
                });
        }
    }

    colors.into_iter().map(|hsl| LighterDarkerResult {
        hsv: Hsv::from_hex(&hsl.as_hex()),
        lightness: hsl.lightness,
        is_original: original_color.as_ref().is_some_and(|oc| 
            (oc.hue - hsl.hue).abs() < FLOAT_TOLERANCE && 
            (oc.saturation - hsl.saturation).abs() < FLOAT_TOLERANCE && 
            (oc.lightness - hsl.lightness).abs() < FLOAT_TOLERANCE
        )
    }).collect()
}
