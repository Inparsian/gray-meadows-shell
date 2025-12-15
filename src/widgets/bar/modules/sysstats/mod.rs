pub mod minimal;
pub mod extended;

use gtk4::prelude::*;

use crate::gesture;
use super::super::module::{BarModule, BarModuleWrapper};

/// Show swap usage only if it's above this threshold, 
/// indicating that the system is under memory pressure.
pub const SWAP_SHOW_THRESHOLD: f64 = 5.0; 

pub fn new() -> BarModuleWrapper {
    let module = BarModule::new(minimal::minimal(), extended::extended());
    let wrapper = BarModuleWrapper::new(module, &["bar-sysstats"]);

    wrapper.bx.add_controller({
        let module = wrapper.module.clone();
        gesture::on_middle_down(move |_, _, _| if !module.is_expanded() {
            let detailed = minimal::DETAILED.get();
            minimal::DETAILED.set(!detailed);
        })
    });

    wrapper
}