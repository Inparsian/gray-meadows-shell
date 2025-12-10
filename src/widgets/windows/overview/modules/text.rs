use std::fmt::Write as _;
use rand::seq::SliceRandom as _;

use super::super::{item::{OverviewSearchItem, OverviewSearchItemAction}, modules::OverviewSearchModule};

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

fn all_operations() -> Vec<TextOperation> {
    use self::TextOperation::*;

    vec![Uppercase, Lowercase, Reverse, Shuffle, Mock, Vaporwave, Uwu, Leet, Rot13, Binary, Hex]
}

fn digest(input: &str, operation: &TextOperation) -> String {
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
                result.push(c);
                
                if c.is_alphabetic() {
                    result.push(' ');
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
                    'i' | 'l' => '1',
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

        TextOperation::Binary => input.chars().fold(String::new(), |mut acc, c| {
            write!(&mut acc, "{:08b}", c as u8).unwrap();
            acc
        }),

        TextOperation::Hex => input.chars().fold(String::new(), |mut acc, c| {
            write!(&mut acc, "{:02x}", c as u8).unwrap();
            acc
        })
    }
}

pub struct OverviewTextModule;

impl OverviewSearchModule for OverviewTextModule {
    fn extensions(&self) -> Vec<&str> {
        vec!["text", "txt", "t", "%"]
    }

    fn icon(&self) -> &str {
        "text_compare"
    }

    fn run(&self, query: &str) -> Vec<OverviewSearchItem> {
        let mut results = Vec::new();

        for operation in all_operations() {
            let result = digest(query, &operation);
            
            if query != result {
                results.push(OverviewSearchItem::new(
                    format!("text-result-{}", format!("{:?}", operation).to_lowercase()),
                    result.clone(),
                    Some(format!("Operation: {}", format!("{:?}", operation).to_lowercase())),
                    "text-x-generic".to_owned(),
                    "copy".to_owned(),
                    OverviewSearchItemAction::Copy(result),
                    None
                ));
            }
        }

        results
    }
}