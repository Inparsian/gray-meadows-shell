use crate::widgets::overview::{item::{OverviewSearchItem, OverviewSearchItemAction}, modules::OverviewSearchModule};

pub struct OverviewTerminalModule;

impl OverviewSearchModule for OverviewTerminalModule {
    fn extensions(&self) -> Vec<&str> {
        vec!["terminal", "term", "cmd", "t", "$"]
    }

    fn run(&self, query: &str) -> Vec<OverviewSearchItem> {
        vec![
            OverviewSearchItem {
                title: query.to_string(),
                subtitle: Some("Run in background".to_string()),
                icon: "utilities-terminal".to_string(),
                action: OverviewSearchItemAction::RunCommand(query.to_string()),
                action_text: "run".to_string(),
                query: None
            },

            OverviewSearchItem {
                title: query.to_string(),
                subtitle: Some("Run in terminal".to_string()),
                icon: "utilities-terminal".to_string(),
                action: OverviewSearchItemAction::Launch(format!("foot fish -C \"{}\"", query.replace('"', "\\\""))),
                action_text: "run".to_string(),
                query: None
            }
        ]
    }
}