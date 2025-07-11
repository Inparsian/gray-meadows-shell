#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum OverviewSearchItemAction {
    Launch(String),
    RunCommand(String),
    Copy(String),
    Custom(fn())
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct OverviewSearchItem {
    pub title: String,
    pub subtitle: Option<String>,
    pub icon: String,
    pub action_text: String,
    pub action: OverviewSearchItemAction,
}