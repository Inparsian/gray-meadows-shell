use std::rc::Rc;
use std::cell::RefCell;
use gtk::prelude::*;

use crate::ffi::astalwp::ffi::{self, Node};
use crate::services::wireplumber;
use crate::utils::gesture;
use crate::widgets::common::dot_separator;

pub struct AudioStream {
    pub node: Node,
    pub root: gtk::Box,
    pub name_label: gtk::Label,
    pub description_label: gtk::Label,
    pub volume_label: gtk::Label,
    pub mute_button: gtk::Button,
    pub is_dragging_volume: Rc<RefCell<bool>>,
    pub volume_slider: gtk::Scale,
}

impl AudioStream {
    pub fn new(node: Node) -> Self {
        let description_label = gtk::Label::new(Some(&node.description));
        description_label.set_css_classes(&["audio-stream-description"]);
        description_label.set_xalign(0.0);
        
        let name_label = gtk::Label::new(Some(&node.name));
        name_label.set_css_classes(&["audio-stream-name"]);
        name_label.set_xalign(0.0);
        name_label.set_hexpand(true);
        name_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        
        let volume_label = gtk::Label::new(Some(&format!("{:.0}%", node.volume * 100.0)));
        volume_label.set_css_classes(&["audio-stream-volume"]);
        volume_label.set_xalign(1.0);
        volume_label.set_width_chars(4);
        volume_label.set_halign(gtk::Align::End);
        
        let mute_button = gtk::Button::new();
        mute_button.set_css_classes(&["audio-stream-mute-button"]);
        mute_button.set_halign(gtk::Align::End);
        mute_button.connect_clicked(move |_| {
            let mute = ffi::node_get_mute(node.id);
            ffi::node_set_mute(node.id, !mute);
        });
        
        if node.mute {
            mute_button.set_label("volume_off");
            mute_button.add_css_class("muted");
        } else {
            mute_button.set_label("volume_up");
        }
        
        let is_dragging_volume = Rc::new(RefCell::new(false));
        let volume_slider = gtk::Scale::new(gtk::Orientation::Horizontal, Some(&gtk::Adjustment::new(0.0, 0.0, 1.0, 0.05, 0.0, 0.0)));
        volume_slider.set_css_classes(&["audio-stream-volume-slider"]);
        volume_slider.set_draw_value(false);
        volume_slider.set_hexpand(true);
        volume_slider.set_value(node.volume as f64);
        volume_slider.connect_value_changed(move |slider| {
            let value = slider.value();
            ffi::node_set_volume(node.id, value as f32);
        });
        
        let volume_slider_drag_gesture = gtk::GestureDrag::new();
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
            move |dy| {
                let current = volume_slider.value();
                let new = dy.mul_add(-0.05, current).clamp(0.0, 1.0);
                ffi::node_set_volume(node.id, new as f32);
            }
        )));
        
        view! {
            root = gtk::Box {
                set_css_classes: &["audio-stream-root"],
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 4,
                
                gtk::Box {
                    append: &description_label,
                    append: &dot_separator::new(),
                    append: &name_label,
                },
                
                gtk::Box {
                    append: &volume_slider,
                    append: &volume_label,
                    append: &mute_button,
                },
            },
        }
        
        Self {
            node,
            root,
            name_label,
            description_label,
            volume_label,
            mute_button,
            is_dragging_volume,
            volume_slider,
        }
    }
    
    pub fn update_from(&mut self, node: &Node) {
        self.node = node.clone();
        self.name_label.set_label(&node.name);
        self.description_label.set_label(&node.description);
        self.volume_label.set_label(&format!("{:.0}%", node.volume * 100.0));
        
        if node.mute {
            self.mute_button.set_label("volume_off");
            self.mute_button.add_css_class("muted");
        } else {
            self.mute_button.set_label("volume_up");
            self.mute_button.remove_css_class("muted");
        }
        
        if !*self.is_dragging_volume.borrow() {
            self.volume_slider.set_value(node.volume as f64);
        }
    }
}

pub struct AudioStreams {
    pub streams: Rc<RefCell<Vec<AudioStream>>>,
    pub bx: gtk::Box,
    pub root: gtk::ScrolledWindow,
}

impl Default for AudioStreams {
    fn default() -> Self {
        let bx = gtk::Box::new(gtk::Orientation::Vertical, 4);
        bx.set_css_classes(&["audio-streams-root"]);
        
        let root = gtk::ScrolledWindow::new();
        root.set_child(Some(&bx));
        root.set_hscrollbar_policy(gtk::PolicyType::Never);
        root.set_vscrollbar_policy(gtk::PolicyType::Automatic);
        root.set_vexpand(true);
        root.set_min_content_height(100);
        
        Self {
            streams: Rc::new(RefCell::new(Vec::new())),
            bx,
            root,
        }
    }
}

impl AudioStreams {
    pub fn add_stream(&self, stream: Node) {
        let stream = AudioStream::new(stream);
        self.bx.append(&stream.root);
        self.streams.borrow_mut().push(stream);
    }
    
    pub fn remove_stream(&self, stream: &Node) {
        let Some(index) = self.streams.borrow().iter().position(|s| s.node.id == stream.id) else {
            return;
        };
        
        let stream = self.streams.borrow_mut().remove(index);
        self.bx.remove(&stream.root);
    }
    
    pub fn update_stream(&self, id: i32) {
        let Some(index) = self.streams.borrow().iter().position(|s| s.node.id == id) else {
            return;
        };
        
        let Some(node) = wireplumber::get_node(id) else {
            return;
        };
        
        self.streams.borrow_mut()[index].update_from(&node);
    }
}