pub mod minimal;
pub mod extended;

use gtk4::prelude::*;

use crate::utils::gesture;
use super::super::base::BarModule;

/// Show swap usage only if it's above this threshold, 
/// indicating that the system is under memory pressure.
pub const SWAP_SHOW_THRESHOLD: f64 = 5.0; 

pub fn new() -> BarModule {
    let module = BarModule::with_widgets(&minimal::minimal(), &extended::extended());
    module.add_css_class("bar-sysstats");

    module.add_controller(gesture::on_middle_down(clone!(
        #[weak] module,
        move |_, _, _| if !module.expanded() {
            let detailed = minimal::DETAILED.get();
            minimal::DETAILED.set(!detailed);
        }
    )));

    module
}