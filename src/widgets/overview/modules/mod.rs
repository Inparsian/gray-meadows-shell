use crate::widgets::overview::item::OverviewSearchItem;

pub mod calculator;
pub mod text;
pub mod terminal;
pub mod hashing;

pub trait OverviewSearchModule {
    fn extensions(&self) -> Vec<&str>;
    fn run(&self, query: &str) -> Vec<OverviewSearchItem>;
}

pub fn validate_input(module: &dyn OverviewSearchModule, query: &str) -> bool {
    module.extensions().iter().any(|ext| query.starts_with(&format!("{} ", ext)))
}

pub fn input_without_extensions(module: &dyn OverviewSearchModule, query: &str) -> String {
    // We have to go through the longest extensions first. For instance, if we remove
    // 'h' before 'hash', the long 'hash' extension won't work because it will be replaced
    // with 'ash', which does not exactly match 'hash'.
    let sorted_exts = {
        let mut exts = module.extensions();
        exts.sort_by_key(|b| std::cmp::Reverse(b.len()));
        exts
    };

    for ext in sorted_exts {
        if query.starts_with(ext) {
            return query.strip_prefix(ext).unwrap_or(query).trim().to_owned();
        }
    }
    
    query.to_owned()
}