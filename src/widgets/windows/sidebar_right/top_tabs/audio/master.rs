use std::rc::Rc;
use std::cell::RefCell;
use gtk4::prelude::*;
use gtk4::glib::{self, clone};

use crate::ffi::astalwp::ffi::{self, Endpoint, EndpointType};
use crate::singletons::wireplumber;
use crate::utils::gesture;

pub struct MasterControls {
    pub root: gtk4::Box,
    pub is_dragging_volume: Rc<RefCell<bool>>,
    pub volume_slider: gtk4::Scale,
    pub volume_label: gtk4::Label,
}

impl MasterControls {
    pub fn new(type_: EndpointType) -> Self {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.set_css_classes(&["audio-master-controls"]);
        
        let controls_bx = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
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
        
        Self {
            root,
            is_dragging_volume,
            volume_slider,
            volume_label,
        }
    }
    
    pub fn update_from(&self, endpoint: &Endpoint) {
        self.volume_label.set_text(&format!("{:.0}%", endpoint.node.volume * 100.0));
        if !*self.is_dragging_volume.borrow() {
            self.volume_slider.set_value(endpoint.node.volume as f64);
        }
    }
}