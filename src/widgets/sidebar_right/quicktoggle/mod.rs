pub mod keybinds;

use futures_signals::signal::Mutable;
use gtk4::prelude::*;

#[derive(Clone)]
pub struct QuickToggleMuiIcon {
    pub enabled: String, // mui icon name
    pub disabled: String, // mui icon name for disabled state
}

impl QuickToggleMuiIcon {
    pub fn new(enabled: &str, disabled: &str) -> Self {
        Self {
            enabled: enabled.to_owned(),
            disabled: disabled.to_owned(),
        }
    }
}

pub struct QuickToggle {
    pub button: gtk4::Button,
    pub toggled: Mutable<bool>,
    pub icon: QuickToggleMuiIcon,
    pub label: Option<gtk4::Label>, // if icon is set then this should be as well
}

pub fn get_css_classes(toggled: bool) -> Vec<&'static str> {
    let mut classes = vec!["sidebar-right-quicktoggle-button"];
    if toggled {
        classes.push("toggled");
    }
    classes
}

pub fn gen_button_with_mui_icon(
    label: &gtk4::Label,
    icon: &QuickToggleMuiIcon,
    toggled: Mutable<bool>,
    callback: Option<Box<dyn Fn(bool) -> bool>>
) -> gtk4::Button {
    let button = gtk4::Button::new();
    button.set_css_classes(get_css_classes(false).as_slice());
    button.set_halign(gtk4::Align::End);
    button.set_valign(gtk4::Align::Center);
    button.connect_clicked({
        let button = button.clone();
        let label = label.clone();
        let icon = icon.clone();
        move |_| {
            if let Some(cb) = &callback {
                toggled.set(cb(toggled.get()));
                button.set_css_classes(get_css_classes(toggled.get()).as_slice());
                label.set_label(if toggled.get() { &icon.enabled } else { &icon.disabled });
            }
        }
    });

    button.set_child(Some(label));

    button
}

impl QuickToggle {
    pub fn new_from_icon(icon: QuickToggleMuiIcon, callback: Option<Box<dyn Fn(bool) -> bool>>) -> Self {
        let toggled = Mutable::new(false);
        let mui_icon_label = gtk4::Label::new(None);
        mui_icon_label.set_label(&icon.enabled);
        mui_icon_label.set_xalign(0.5);
        mui_icon_label.set_halign(gtk4::Align::Center);

        let button = gen_button_with_mui_icon(&mui_icon_label, &icon, toggled.clone(), callback);

        Self {
            button,
            label: Some(mui_icon_label),
            toggled,
            icon
        }
    }

    /// You would set this quick toggle's toggle state in the callback. However, if it has to be
    /// set directly (i.e. from an external event), use this method.
    pub fn set_toggled(&self, toggled: bool) {
        self.toggled.set(toggled);
        self.button.set_css_classes(get_css_classes(toggled).as_slice());
        
        if let Some(label) = &self.label {
            label.set_label(if toggled { &self.icon.enabled } else { &self.icon.disabled });
        }
    }
}