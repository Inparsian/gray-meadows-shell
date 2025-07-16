use crate::widgets::overview::{item::{OverviewSearchItem, OverviewSearchItemAction}, modules::OverviewSearchModule};

pub struct OverviewTerminalModule;

impl OverviewSearchModule for OverviewTerminalModule {
    fn extensions(&self) -> Vec<&str> {
        vec!["terminal", "term", "cmd", "t", "$"]
    }

    fn run(&self, query: &str) -> Vec<OverviewSearchItem> {
        vec![
            OverviewSearchItem {
                title: query.to_owned(),
                subtitle: Some("Run in background".to_owned()),
                icon: "utilities-terminal".to_owned(),
                action: OverviewSearchItemAction::RunCommand(query.to_owned()),
                action_text: "run".to_owned(),
                query: None
            },

            OverviewSearchItem {
                title: query.to_owned(),
                subtitle: Some("Run in terminal".to_owned()),
                icon: "utilities-terminal".to_owned(),
                action: OverviewSearchItemAction::Launch(format!("foot fish -C \"{}\"", query.replace('"', "\\\""))),
                action_text: "run".to_owned(),
                query: None
            }
        ]
    }
}