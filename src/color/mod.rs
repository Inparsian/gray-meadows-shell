pub mod models;

use std::cmp::Ordering::Equal;

use crate::FLOAT_TOLERANCE;
use self::models::{Rgba, Hsv, Hsl, Cmyk, Oklch, ColorModel as _};

pub struct LighterDarkerResult {
    pub hsv: Hsv,
    pub lightness: f64,
    pub is_original: bool,
}

pub fn is_valid_hex_color(hex: &str) -> bool {
    let hex = hex.trim_start_matches('#');
    let len = hex.len();

    (len == 6 || len == 8 || len == 3 || len == 4) && hex.chars().all(|c| c.is_ascii_hexdigit())
}

pub fn hex_to_int(hex: &str) -> u32 {
    u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap_or(0)
}

pub fn int_to_hex(int: u32) -> String {
    format!("#{:06x}", int)
}

#[allow(clippy::unreadable_literal)]
pub fn get_int_color(int_str: &str) -> Option<u32> {
    int_str.trim().parse::<u32>().ok().filter(|&value| value <= 0xFFFFFF)
}

pub fn parse_color_into_hex(string: &str) -> Option<String> {
    [
        |s: &str| (s.starts_with('#') && is_valid_hex_color(s)).then(|| s.to_owned()),
        |s: &str| get_int_color(s).map(int_to_hex),
        |s: &str| Rgba::from_string(s).map(|rgba| rgba.into_hex()),
        |s: &str| Hsv::from_string(s).map(|hsv| hsv.into_hex()),
        |s: &str| Hsl::from_string(s).map(|hsl| hsl.into_hex()),
        |s: &str| Cmyk::from_string(s).map(|cmyk| cmyk.into_hex()),
        |s: &str| Oklch::from_string(s).map(|oklch| oklch.into_hex()),
    ]
    .iter().find_map(|parse| parse(string))
}

pub fn get_analogous_colors(hsv: Hsv, count: u32) -> Vec<Hsv> {
    let step = 360.0 / count as f64;
    let mut colors = Vec::new();

    for i in 0..count {
        let mut new_hsv = hsv;
        new_hsv.hue = (i as f64).mul_add(step, hsv.hue) % 360.0;
        colors.push(new_hsv);
    }

    colors
}

pub fn get_lighter_darker_colors(base_hsv: Hsv, count: u32) -> Vec<LighterDarkerResult> {
    let base_hsl = Hsl::from_model(base_hsv);
    let l_comp = |a: &Hsl, b: &Hsl, c: &Hsl| {
        a.l_diff(c).partial_cmp(&b.l_diff(c)).unwrap_or(Equal)
    };
    let hsl_below_tolerance = |a: &Hsl, b: &Hsl| {
        a.h_diff(b) < FLOAT_TOLERANCE && 
        a.s_diff(b) < FLOAT_TOLERANCE && 
        a.l_diff(b) < FLOAT_TOLERANCE
    };

    let mut colors: Vec<Hsl> = vec![base_hsl];

    for i in 0..=count {
        let mut lighter_hsl = base_hsl;
        lighter_hsl.lightness = (i as f64).mul_add(-(100.0 / count as f64), 100.0);
        colors.push(lighter_hsl);
    }

    colors.sort_by(|a, b| b.lightness.partial_cmp(&a.lightness).unwrap_or(Equal));
    colors.dedup_by(|a, b| hsl_below_tolerance(a, b));

    // Whatever color is closest to the original color, mark it as the original
    let original_color = colors.iter().min_by(|a, b| l_comp(a, b, &base_hsl)).copied();

    // If the original lightness is divisible by exactly 5, it'll have been
    // deduped already at this point.
    if let Some(original) = original_color 
        && (original.lightness % 5.0) >= FLOAT_TOLERANCE
        && let Some((index, _)) = colors.iter().enumerate()
            .filter(|(_, c)| (c.lightness % 5.0).abs() <= FLOAT_TOLERANCE)
            .min_by(|(_, a), (_, b)| l_comp(a, b, &original))
    {
        colors.remove(index);
    }

    colors.into_iter().map(|hsl| LighterDarkerResult {
        hsv: Hsv::from_model(hsl),
        lightness: hsl.lightness,
        is_original: original_color.as_ref().is_some_and(|oc| hsl_below_tolerance(oc, &hsl))
    })
    .collect()
}
