pub mod model;

use std::cmp::Ordering::Equal;

use crate::FLOAT_TOLERANCE;
use self::model::{Rgba, Hsv, Hsl, Cmyk, Oklch, int_to_hex};

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

#[allow(clippy::unreadable_literal)]
pub fn get_int_color(int_str: &str) -> Option<u32> {
    int_str.trim().parse::<u32>().ok().filter(|&value| value <= 0xFFFFFF)
}

pub fn parse_color_into_hex(string: &str) -> Option<String> {
    [
        |s: &str| (s.starts_with('#') && is_valid_hex_color(s)).then(|| s.to_owned()),
        |s: &str| get_int_color(s).map(int_to_hex),
        |s: &str| Rgba::from_string(s).map(|rgba| rgba.as_hex()),
        |s: &str| Hsv::from_string(s).map(|hsv| hsv.as_hex()),
        |s: &str| Hsl::from_string(s).map(|hsl| hsl.as_hex()),
        |s: &str| Cmyk::from_string(s).map(|cmyk| cmyk.as_hex()),
        |s: &str| Oklch::from_string(s).map(|oklch| oklch.as_hex()),
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
    let base_hsl = base_hsv.as_hsl();
    let hsl_below_tolerance = |a: &model::Hsl, b: &model::Hsl| {
        a.h_diff(b) < FLOAT_TOLERANCE && 
        a.s_diff(b) < FLOAT_TOLERANCE && 
        a.l_diff(b) < FLOAT_TOLERANCE
    };

    let mut colors: Vec<model::Hsl> = vec![base_hsl];

    for i in 0..=count {
        let mut lighter_hsl = base_hsl;
        lighter_hsl.lightness = (i as f64).mul_add(-(100.0 / count as f64), 100.0);
        colors.push(lighter_hsl);
    }

    colors.sort_by(|a, b| b.lightness.partial_cmp(&a.lightness).unwrap_or(Equal));
    colors.dedup_by(|a, b| hsl_below_tolerance(a, b));

    // Whatever color is closest to the original color, mark it as the original
    let original_color = colors.iter().min_by(|a, b| {
        a.l_diff(&base_hsl).partial_cmp(&b.l_diff(&base_hsl)).unwrap_or(Equal)
    }).copied();

    // If the original lightness is divisible by exactly 5, it'll have been
    // deduped already at this point.
    if let Some(original) = original_color {
        if (original.lightness % 5.0) >= FLOAT_TOLERANCE {
            if let Some((index, _)) = colors.iter().enumerate()
                .filter(|(_, c)| (c.lightness % 5.0).abs() <= FLOAT_TOLERANCE)
                .min_by(|(_, a), (_, b)| a.l_diff(&original).partial_cmp(&b.l_diff(&original)).unwrap_or(Equal))
            {
                colors.remove(index);
            }
        }
    }

    colors.into_iter().map(|hsl| LighterDarkerResult {
        hsv: Hsv::from_hex(&hsl.as_hex()),
        lightness: hsl.lightness,
        is_original: original_color.as_ref().is_some_and(|oc| hsl_below_tolerance(oc, &hsl))
    })
    .collect()
}
