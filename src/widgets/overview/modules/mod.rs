#[allow(dead_code)]
pub trait OverviewSearchModule {
    fn extensions(&self) -> Vec<String>;
    fn run(&self, query: &str);
}