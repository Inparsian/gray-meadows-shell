mod minimal;
mod extended;

use crate::widgets::bar::module::{BarModule, BarModuleWrapper};

pub fn new() -> BarModuleWrapper {
    let module = BarModule::new(minimal::minimal(), extended::extended());
    BarModuleWrapper::new(module, &["bar-mpris"])
}