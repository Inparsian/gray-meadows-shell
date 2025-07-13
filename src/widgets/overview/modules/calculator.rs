use crate::{
    ffi::libqalculate::ffi,
    widgets::overview::{item::{OverviewSearchItem, OverviewSearchItemAction}, modules::OverviewSearchModule}
};

pub struct OverviewCalculatorModule;

impl OverviewSearchModule for OverviewCalculatorModule {
    fn extensions(&self) -> Vec<&str> {
        vec!["calc", "qalc", "c", "="]
    }

    fn run(&self, query: &str) -> Vec<OverviewSearchItem> {
        let unlocalized = ffi::unlocalizeExpression(query.to_string());
        let result = ffi::calculateAndPrint(unlocalized, 1000);

        vec![OverviewSearchItem {
            title: result.clone(),
            subtitle: Some("Math result".to_string()),
            icon: "accessories-calculator".to_string(),
            action: OverviewSearchItemAction::Copy(result),
            action_text: "calculate".to_string(),
            query: None
        }]
    }
}