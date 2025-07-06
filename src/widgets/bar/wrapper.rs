use gtk4::prelude::*;

pub struct BarModuleWrapper {
    bx: gtk4::Box,
}

impl BarModuleWrapper {
    pub fn new(inner_bx: gtk4::Box) -> Self {
        let bx = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        bx.set_css_classes(&["bar-widget-wrapper"]);
        bx.set_hexpand(false);
        bx.append(&inner_bx);

        Self {
            bx
        }
    }

    pub fn add_controller(self, controller: impl IsA<gtk4::EventController>) -> Self {
        self.bx.add_controller(controller);
        self
    }

    pub fn get_widget(self) -> gtk4::Box {
        self.bx
    }
}