use std::rc::Rc;
use std::cell::RefCell;
use gtk4::prelude::*;

use crate::ffi::astalwp::ffi::{self, Endpoint, EndpointType};
use crate::services::wireplumber;
use crate::utils::gesture;
use crate::widgets::common::revealer::{AdwRevealer, AdwRevealerDirection, GEasing};

pub struct MasterDevice {
    pub endpoint: Endpoint,
    pub root: gtk4::Button,
    pub default_icon: gtk4::Label,
}

impl MasterDevice {
    pub fn new(endpoint: Endpoint) -> Self {
        let root = gtk4::Button::new();
        root.set_css_classes(&["audio-master-device-root"]);
        root.connect_clicked(move |_| {
            ffi::endpoint_set_is_default(endpoint.node.id, true);
        });
        
        let root_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        root_box.set_css_classes(&["audio-master-device-box"]);
        root.set_child(Some(&root_box));
        
        let device_icon = gtk4::Label::new(Some(match endpoint.type_ {
            EndpointType::Microphone => "mic",
            _ => "speaker",
        }));
        device_icon.set_css_classes(&["audio-master-device-icon"]);
        root_box.append(&device_icon);
        
        let device_name = gtk4::Label::new(Some(&endpoint.node.description));
        device_name.set_css_classes(&["audio-master-device-name"]);
        device_name.set_xalign(0.0);
        device_name.set_hexpand(true);
        device_name.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        root_box.append(&device_name);
        
        let default_icon = gtk4::Label::new(Some("check"));
        default_icon.set_css_classes(&["audio-master-device-default-icon"]);
        default_icon.set_visible(endpoint.is_default);
        default_icon.set_xalign(1.0);
        default_icon.set_halign(gtk4::Align::End);
        root_box.append(&default_icon);
        
        Self {
            endpoint,
            root,
            default_icon,
        }
    }
    
    pub fn set_default(&self, is_default: bool) {
        self.default_icon.set_visible(is_default);
    }
}

pub struct MasterControls {
    pub root: gtk4::Box,
    pub endpoints: Rc<RefCell<Vec<MasterDevice>>>,
    pub devices_box: gtk4::Box,
    pub is_dragging_volume: Rc<RefCell<bool>>,
    pub volume_slider: gtk4::Scale,
    pub volume_label: gtk4::Label,
}

impl MasterControls {
    pub fn new(type_: EndpointType) -> Self {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.set_css_classes(&["audio-master-root"]);
        
        let controls_bx = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        controls_bx.set_css_classes(&["audio-master-controls"]);
        root.append(&controls_bx);
        
        let master_icon = gtk4::Label::new(Some(match type_ {
            EndpointType::Microphone => "mic",
            _ => "speaker",
        }));
        master_icon.add_css_class("audio-master-icon");
        controls_bx.append(&master_icon);
        
        let is_dragging_volume = Rc::new(RefCell::new(false));
        let volume_slider = gtk4::Scale::new(gtk4::Orientation::Horizontal, Some(&gtk4::Adjustment::new(0.0, 0.0, 1.0, 0.05, 0.0, 0.0)));
        volume_slider.set_css_classes(&["audio-master-volume-slider"]);
        volume_slider.set_draw_value(false);
        volume_slider.set_hexpand(true);
        volume_slider.set_value(0.0);
        volume_slider.connect_value_changed(clone!(
            #[strong] type_,
            move |slider| {
                let value = slider.value();
                if let Some(endpoint) = match type_ {
                    EndpointType::Microphone => wireplumber::get_default_microphone(),
                    _ => wireplumber::get_default_speaker(),
                } {
                    ffi::node_set_volume(endpoint.node.id, value as f32);
                }
            }
        ));
        
        let volume_slider_drag_gesture = gtk4::GestureDrag::new();
        volume_slider_drag_gesture.connect_drag_begin(clone!(
            #[strong] is_dragging_volume,
            move |_, _, _| {
                *is_dragging_volume.borrow_mut() = true;
            }
        ));
        volume_slider_drag_gesture.connect_drag_end(clone!(
            #[strong] is_dragging_volume,
            move |_, _, _| {
                *is_dragging_volume.borrow_mut() = false;
            }
        ));
        volume_slider.add_controller(volume_slider_drag_gesture);
        volume_slider.add_controller(gesture::on_vertical_scroll(clone!(
            #[weak] volume_slider,
            #[strong] type_,
            move |dy| {
                let current = volume_slider.value();
                let new = dy.mul_add(-0.05, current).clamp(0.0, 1.0);
                if let Some(endpoint) = match type_ {
                    EndpointType::Microphone => wireplumber::get_default_microphone(),
                    _ => wireplumber::get_default_speaker(),
                } {
                    ffi::node_set_volume(endpoint.node.id, new as f32);
                }
            }
        )));
        controls_bx.append(&volume_slider);
        
        let volume_label = gtk4::Label::new(Some("0%"));
        volume_label.set_css_classes(&["audio-master-volume-label"]);
        volume_label.set_xalign(1.0);
        volume_label.set_width_chars(4);
        volume_label.set_halign(gtk4::Align::End);
        controls_bx.append(&volume_label);
        
        let devices_revealer = AdwRevealer::builder()
            .transition_direction(AdwRevealerDirection::Up)
            .transition_duration(300)
            .show_easing(GEasing::EaseOutExpo)
            .hide_easing(GEasing::EaseOutExpo)
            .build();
        root.prepend(&devices_revealer);
        
        let devices_scrolled_window = gtk4::ScrolledWindow::new();
        devices_scrolled_window.set_hscrollbar_policy(gtk4::PolicyType::Never);
        devices_scrolled_window.set_vscrollbar_policy(gtk4::PolicyType::Automatic);
        devices_scrolled_window.set_propagate_natural_height(true);
        devices_scrolled_window.set_min_content_height(108);
        devices_scrolled_window.set_max_content_height(108);
        devices_revealer.set_child(Some(&devices_scrolled_window.clone().upcast()));
        
        let devices_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        devices_box.set_css_classes(&["audio-master-devices-box"]);
        devices_scrolled_window.set_child(Some(&devices_box));
        
        let devices_revealer_button = gtk4::Button::new();
        devices_revealer_button.set_css_classes(&["audio-master-devices-revealer-button"]);
        devices_revealer_button.connect_clicked(clone!(
            #[weak] devices_revealer,
            #[weak] devices_revealer_button,
            move |_| {
                let revealed = !devices_revealer.reveal();
                devices_revealer.set_reveal(revealed);
                
                if revealed {
                    devices_revealer_button.add_css_class("open");
                } else {
                    devices_revealer_button.remove_css_class("open");
                }
            }
        ));
        controls_bx.append(&devices_revealer_button);
        
        let devices_revealer_button_icon = gtk4::Label::new(Some("stat_1"));
        devices_revealer_button_icon.set_css_classes(&["audio-master-devices-revealer-button-icon"]);
        devices_revealer_button.set_child(Some(&devices_revealer_button_icon));
        
        Self {
            root,
            endpoints: Rc::new(RefCell::new(Vec::new())),
            devices_box,
            is_dragging_volume,
            volume_slider,
            volume_label,
        }
    }
    
    pub fn add_device(&self, endpoint: Endpoint) {
        let device = MasterDevice::new(endpoint);
        self.devices_box.append(&device.root);
        self.endpoints.borrow_mut().push(device);
    }
    
    pub fn remove_device(&self, endpoint: &Endpoint) {
        let Some(index) = self.endpoints.borrow().iter().position(|e| e.endpoint.node.id == endpoint.node.id) else {
            return;
        };
        
        let device = self.endpoints.borrow_mut().remove(index);
        self.devices_box.remove(&device.root);
    }
    
    pub fn update_from(&self, endpoint: &Endpoint, refresh_defaults: bool) {
        if endpoint.is_default {
            self.volume_label.set_text(&format!("{:.0}%", endpoint.node.volume * 100.0));
            if !*self.is_dragging_volume.borrow() {
                self.volume_slider.set_value(endpoint.node.volume as f64);
            }
            
            if refresh_defaults {
                for device in self.endpoints.borrow().iter() {
                    device.set_default(device.endpoint.node.id == endpoint.node.id);
                }
            }
        }
    }
}