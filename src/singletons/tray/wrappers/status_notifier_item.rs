#[derive(Clone, Debug)]
pub struct StatusNotifierItem {
    pub owner: String
}

impl StatusNotifierItem {
    pub fn new(owner: String) -> Self {
        Self { owner }
    }
}