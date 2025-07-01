use gtk4::prelude::*;

#[allow(dead_code)]
#[derive(Default, Clone)]
struct SystemTrayItem {
    pub owner: String,
    pub widget: Option<gtk4::Image>,
    popover_menu: Option<gtk4::PopoverMenu>
}

#[allow(dead_code)]
impl SystemTrayItem {
    pub fn new(owner: String) -> Self {
        Self {
            owner,
            ..Self::default()
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
struct SystemTray {
    box_: gtk4::Box,
    items: Vec<SystemTrayItem>,
}

impl SystemTray {
    fn new() -> Self {
        let box_ = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        box_.set_css_classes(&["bar-widget", "bar-tray"]);
        box_.set_hexpand(false);

        let items = Vec::new();

        Self {
            box_,
            items,
        }
    }

    fn get_widget(&self) -> gtk4::Box {
        self.box_.clone()
    }
}

pub fn new() -> gtk4::Box {
    let tray = SystemTray::new();
    
    tray.get_widget()
}