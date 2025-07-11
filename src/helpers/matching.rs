#![allow(dead_code)]

pub fn lazy_match(haystack: &str, needle: &str) -> bool {
    let mut needle_chars: Vec<char> = needle.chars().collect();
    needle_chars.dedup();

    for c in haystack.chars() {
        if let Some(index) = needle_chars.iter().position(|&x| x == c) {
            needle_chars.remove(index);
        }
    }

    needle_chars.is_empty()
}

pub fn lazy_match_indices(haystack: &str, needle: &str) -> Vec<(usize, usize)> {
    let mut needle_chars: Vec<char> = needle.chars().collect();
    needle_chars.dedup();

    let mut indices: Vec<(usize, usize)> = Vec::new();

    for (i, c) in haystack.chars().enumerate() {
        if let Some(index) = needle_chars.iter().position(|&x| x == c) {
            indices.push((i, index));
            needle_chars.remove(index);
        }
    }

    indices
}

pub fn fuzzy_match(haystack: &str, needle: &str) -> bool {
    let mut needle = needle.chars();

    for c in haystack.chars() {
        if let Some(n) = needle.clone().next() {
            if c == n {
                needle.next();
            }
        }
    }

    needle.next().is_none()
}