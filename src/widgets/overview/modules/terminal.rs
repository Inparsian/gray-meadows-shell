use crate::widgets::overview::{item::{OverviewSearchItem, OverviewSearchItemAction}, modules::OverviewSearchModule};

pub struct OverviewTerminalModule;

impl OverviewSearchModule for OverviewTerminalModule {
    fn extensions(&self) -> Vec<&str> {
        vec!["terminal", "term", "cmd", "t", "$"]
    }

    fn icon(&self) -> &str {
        "terminal"
    }

    fn run(&self, query: &str) -> Vec<OverviewSearchItem> {
        vec![
            OverviewSearchItem::new(
                "command-run-in-background".to_owned(),
                query.to_owned(),
                Some("Run in background".to_owned()),
                "utilities-terminal".to_owned(),
                "run".to_owned(),
                OverviewSearchItemAction::RunCommand(query.to_owned()),
                None
            ),

            OverviewSearchItem::new(
                "command-run-in-terminal".to_owned(),
                query.to_owned(),
                Some("Run in terminal".to_owned()),
                "utilities-terminal".to_owned(),
                "run".to_owned(),
                OverviewSearchItemAction::Launch(format!("foot fish -C \"{}\"", query.replace('"', "\\\""))),
                None
            )
        ]
    }
}