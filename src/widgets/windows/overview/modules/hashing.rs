use std::fmt::Write as _;
use md5;
use sha::{self, utils::{Digest as _, DigestExt as _}};
use whirlpool::{Whirlpool, Digest as _};

use super::super::{item::{OverviewSearchItem, OverviewSearchItemAction}, modules::OverviewSearchModule};

#[derive(Debug, Clone)]
enum Algorithm {
    Md5,
    Sha1,
    Sha224,
    Sha256,
    Sha384,
    Sha512,
    Whirlpool
}

fn all_algorithms() -> Vec<Algorithm> {
    use self::Algorithm::*;

    vec![Md5, Sha1, Sha224, Sha256, Sha384, Sha512, Whirlpool]
}

fn digest(input: &str, algorithm: &Algorithm) -> String {
    match algorithm {
        Algorithm::Md5 => format!("{:x}", md5::compute(input)),
        Algorithm::Sha1 => sha::sha1::Sha1::default().digest(input.as_bytes()).to_hex(),
        Algorithm::Sha224 => sha::sha224::Sha224::default().digest(input.as_bytes()).to_hex(),
        Algorithm::Sha256 => sha::sha256::Sha256::default().digest(input.as_bytes()).to_hex(),
        Algorithm::Sha384 => sha::sha384::Sha384::default().digest(input.as_bytes()).to_hex(),
        Algorithm::Sha512 => sha::sha512::Sha512::default().digest(input.as_bytes()).to_hex(),
        Algorithm::Whirlpool => {
            let mut hasher = Whirlpool::new();
            hasher.update(input.as_bytes());
            hasher.finalize().to_vec().iter().fold(String::new(), |mut acc, b| {
                write!(&mut acc, "{:02x}", b).unwrap();
                acc
            })
        }
    }
}

pub struct OverviewHashingModule;

impl OverviewSearchModule for OverviewHashingModule {
    fn extensions(&self) -> Vec<&str> {
        vec!["hash", "h", "#"]
    }

    fn icon(&self) -> &str {
        "tag"
    }

    fn run(&self, query: &str) -> Vec<OverviewSearchItem> {
        let mut results = Vec::new();

        for algorithm in all_algorithms() {
            let result = digest(query, &algorithm);
            
            if query != result {
                results.push(OverviewSearchItem::new(
                    format!("hash-result-{}", format!("{:?}", algorithm).to_lowercase()),
                    result.clone(),
                    Some(format!("Operation: {}", format!("{:?}", algorithm).to_lowercase())),
                    "hashit".to_owned(),
                    "copy".to_owned(),
                    OverviewSearchItemAction::Copy(result),
                    None
                ));
            }
        }

        results
    }
}