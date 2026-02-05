use std::cell::RefCell;
use gtk::prelude::*;
use relm4::RelmRemoveAllExt as _;

use crate::services::weather::schemas::nws::{NwsAlertsResponse, NwsFeature};

#[derive(Clone)]
pub struct WeatherAlert {
    pub alert: NwsFeature,
    pub bx: gtk::Box,
}

impl WeatherAlert {
    pub fn new(alert: NwsFeature) -> Self {
        let bx = gtk::Box::new(gtk::Orientation::Vertical, 0);
        bx.set_hexpand(true);

        Self {
            alert,
            bx,
        }
    }
    
    pub fn construct_field(label: &str, value: &str) -> gtk::Box {
        view! {
            bx = gtk::Box {
                set_css_classes: &["weather-alert-field"],
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 4,
                
                gtk::Label {
                    set_label: label,
                    set_css_classes: &["weather-alert-field-label"],
                    set_hexpand: false,
                    set_xalign: 0.0,
                },
                
                gtk::Label {
                    set_label: value,
                    set_css_classes: &["weather-alert-field-value"],
                    set_hexpand: true,
                    set_xalign: 1.0,
                    set_wrap: true,
                },
            },
        }

        bx
    }
    
    pub fn construct(&self) {
        let parse_dt = |dt: Option<&String>| dt.map_or_else(
            || "indeterminate".to_owned(),
            |dt| chrono::DateTime::parse_from_rfc3339(dt)
                .map_or_else(|_| dt.to_owned(), |dt| dt.format("%B %d, %Y at %I:%M %p").to_string())
        );
        
        view! {
            revealer = gtk::Revealer {
                set_hexpand: true,
                set_reveal_child: false,
                set_transition_type: gtk::RevealerTransitionType::SlideDown,
                set_transition_duration: 175,
                
                gtk::Box {
                    set_css_classes: &["weather-alert-details"],
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 4,
                    
                    append: &Self::construct_field("Sent by", &self.alert.properties.sender_name),
                    append: &Self::construct_field("Sent on", &parse_dt(Some(&self.alert.properties.sent))),
                    append: &Self::construct_field("Effective", &parse_dt(Some(&self.alert.properties.effective))),
                    append: &Self::construct_field("Onset", &parse_dt(self.alert.properties.onset.as_ref())),
                    append: &Self::construct_field("Expires", &parse_dt(Some(&self.alert.properties.expires))),
                    append: &Self::construct_field("Ends", &parse_dt(self.alert.properties.ends.as_ref())),
                    
                    gtk::Box {
                        set_margin_top: 16,
                        set_css_classes: &["weather-alert-description"],
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 16,
                        
                        gtk::Label {
                            set_label: &self.alert.properties.description,
                            set_css_classes: &["weather-alert-description"],
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                        },
                        
                        gtk::Label {
                            set_label: &self.alert.properties.instruction.as_ref().unwrap_or(&"...".to_owned()),
                            set_css_classes: &["weather-alert-instruction"],
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                        },
                    }
                },
            },
            
            button = gtk::Button {
                set_css_classes: &["weather-alert-button"],
                set_hexpand: true,
                connect_clicked: clone!(
                    #[weak] revealer,
                    move |_| revealer.set_reveal_child(!revealer.reveals_child())
                ),
                
                gtk::Box {
                    set_spacing: 4,

                    gtk::Label {
                        set_label: "warning",
                        set_css_classes: &["weather-alert-icon"],
                    },

                    gtk::Label {
                        set_label: &self.alert.properties.event,
                        set_hexpand: true,
                        set_xalign: 0.0,
                        set_max_width_chars: 1,
                        set_wrap: true,
                        set_css_classes: &["weather-alert-event"],
                    },
                },
            },
        }

        self.bx.remove_all();
        self.bx.append(&button);
        self.bx.append(&revealer);
    }
}

pub struct WeatherAlerts {
    pub bx: gtk::Box,
    pub root: gtk::ScrolledWindow,
    pub alerts: RefCell<Vec<WeatherAlert>>,
}

impl Default for WeatherAlerts {
    fn default() -> Self {
        let bx = gtk::Box::new(gtk::Orientation::Vertical, 4);
        bx.set_css_classes(&["weather-alerts"]);
        
        let root = gtk::ScrolledWindow::new();
        root.set_child(Some(&bx));
        root.set_vexpand(true);
        root.set_hexpand(true);
        root.set_hscrollbar_policy(gtk::PolicyType::Never);

        Self {
            bx,
            root,
            alerts: RefCell::new(Vec::new()),
        }
    }
}

impl WeatherAlerts {
    pub fn update(&self, alerts: &NwsAlertsResponse) {
        let mut alerts_mut = self.alerts.borrow_mut();
        for alert in alerts_mut.clone() {
            if !alerts.features.iter().any(|a| a.properties.id == alert.alert.properties.id) {
                self.bx.remove(&alert.bx);
                alerts_mut.retain(|a| a.alert.properties.id != alert.alert.properties.id);
            }
        }
    
        for alert in &alerts.features {
            if !alerts_mut.iter().any(|a| a.alert.properties.id == alert.properties.id) {
                let alert = WeatherAlert::new(alert.clone());
                alerts_mut.push(alert.clone());
                self.bx.append(&alert.bx);
                alert.construct();
            }
        }
    }
}