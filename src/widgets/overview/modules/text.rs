use rand::seq::SliceRandom;

use crate::widgets::overview::{item::{OverviewSearchItem, OverviewSearchItemAction}, modules::OverviewSearchModule};

#[derive(Debug, Clone)]
enum TextOperation {
    Uppercase, // UPPERCASE
    Lowercase, // lowercase
    Reverse, // reverse the string duh
    Shuffle, // shuffle the string
    Mock, // basically random case
    Vaporwave, // space between characters
    Uwu, // uwu
    Leet, // 1337
    Rot13, // ROT13
    Binary, // binary
    Hex // hex
}

fn digest(input: String, operation: TextOperation) -> String {
    match operation {
        TextOperation::Uppercase => input.to_uppercase(),
        TextOperation::Lowercase => input.to_lowercase(),
        TextOperation::Reverse => input.chars().rev().collect::<String>(),

        TextOperation::Shuffle => {
            let mut result = input.chars().collect::<Vec<char>>();
            result.shuffle(&mut rand::rng());

            result.into_iter().collect()
        },

        TextOperation::Mock => {
            let mut result = String::new();

            for c in input.chars() {
                if c.is_alphabetic() {
                    if rand::random() {
                        result.push(c.to_uppercase().next().unwrap());
                    } else {
                        result.push(c.to_lowercase().next().unwrap());
                    }
                } else {
                    result.push(c);
                }
            }

            result
        },

        TextOperation::Vaporwave => {
            let mut result = String::new();

            for c in input.chars() {
                if c.is_alphabetic() {
                    result.push(c);
                    result.push(' ');
                } else {
                    result.push(c);
                }
            }

            result
        },

        TextOperation::Uwu => {
            let mut result = String::new();

            for c in input.chars() {
                result.push(match c {
                    'r' | 'l' => 'w',
                    'R' | 'L' => 'W',
                    _ => c
                });
            }

            result
        },

        TextOperation::Leet => {
            let mut result = String::new();

            for c in input.chars() {
                result.push(match c.to_ascii_lowercase() {
                    'a' => '4',
                    'e' => '3',
                    'i' => '1',
                    'l' => '1',
                    'o' => '0',
                    's' => '5',
                    't' => '7',
                    _ => c
                });
            }

            result
        },

        TextOperation::Rot13 => {
            input.chars().map(|c| {
                if c.is_ascii_alphabetic() {
                    let first = if c.is_ascii_lowercase() { 
                        b'a' 
                    } else { 
                        b'A' 
                    };

                    (((c as u8 - first + 13) % 26) + first) as char
                } else {
                    c
                }
            }).collect()
        },

        TextOperation::Binary => input.chars().map(|c| format!("{:08b}", c as u8)).collect::<String>(),
        TextOperation::Hex => input.chars().map(|c| format!("{:02x}", c as u8)).collect::<String>()
    }
}

pub struct OverviewTextModule;

impl OverviewSearchModule for OverviewTextModule {
    fn extensions(&self) -> Vec<&str> {
        vec!["text", "txt", "t", "%"]
    }

    fn run(&self, query: &str) -> Vec<OverviewSearchItem> {
        let mut results = Vec::new();

        for operation in [
            TextOperation::Uppercase,
            TextOperation::Lowercase,
            TextOperation::Reverse,
            TextOperation::Shuffle,
            TextOperation::Mock,
            TextOperation::Vaporwave,
            TextOperation::Uwu,
            TextOperation::Leet,
            TextOperation::Rot13,
            TextOperation::Binary,
            TextOperation::Hex
        ] {
            let result = digest(query.to_string(), operation.clone());
            
            if query != result {
                results.push(OverviewSearchItem {
                    title: result.clone(),
                    subtitle: Some(format!("Operation: {}", format!("{:?}", operation).to_lowercase())),
                    icon: "text-x-generic".to_string(),
                    action: OverviewSearchItemAction::Copy(result),
                    action_text: "copy".to_string(),
                    query: None
                });
            }
        }

        results
    }
}