use std::cell::RefCell;
use std::rc::Rc;
use gtk::prelude::*;

use crate::config::read_config;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum AudioTargetType {
    App,
    Device,
}

pub struct AudioTarget {
    type_: AudioTargetType,
    name: String,
    pub root: gtk::Box,
}

impl AudioTarget {
    pub fn new(parent: &AudioTargets, type_: AudioTargetType, name: String) -> Self {
        let root = gtk::Box::builder()
            .css_classes(["audio-target"])
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .hexpand(true)
            .build();
        
        root.append(&gtk::Label::builder()
            .label(&name)
            .xalign(0.0)
            .hexpand(true)
            .build());
        
        let remove_button = gtk::Button::builder()
            .css_classes(["audio-target-remove-button"])
            .label("Remove")
            .build();
        remove_button.connect_clicked(clone!(
            #[strong] parent,
            #[strong] name,
            move |_| {
                parent.remove_target(&name);
            }
        ));
        
        root.append(&remove_button);
        
        Self {
            type_,
            name,
            root,
        }
    }
}

#[derive(Clone)]
pub struct AudioTargets {
    type_: AudioTargetType,
    targets_list: gtk::Box,
    targets: Rc<RefCell<Vec<AudioTarget>>>,
    pub root: gtk::Box,
}

impl AudioTargets {
    pub fn new(type_: AudioTargetType) -> Self {
        let root = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(8)
            .hexpand(true)
            .vexpand(true)
            .build();
        
        let field_label = gtk::Label::builder()
            .css_classes(["screen-recorder-field-label"])
            .label(match type_ {
                AudioTargetType::App => "Audio App Targets",
                AudioTargetType::Device => "Audio Device Targets",
            })
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .xalign(0.0)
            .hexpand(true)
            .build();
        
        let targets_list = gtk::Box::builder()
            .css_classes(["audio-targets-list"])
            .orientation(gtk::Orientation::Vertical)
            .spacing(0)
            .hexpand(true)
            .vexpand(true)
            .build();
        
        let targets_list_scrolled = gtk::ScrolledWindow::builder()
            .child(&targets_list)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .hexpand(true)
            .vexpand(true)
            .build();
        
        root.append(&field_label);
        root.append(&targets_list_scrolled);
        
        let me = Self {
            type_,
            targets_list,
            targets: Rc::new(RefCell::new(Vec::new())),
            root,
        };
        
        me.refresh();
        me
    }
    
    pub fn add_target(&self, target_name: &str) {
        let target = AudioTarget::new(
            self,
            self.type_,
            target_name.to_owned(),
        );
        
        self.targets_list.append(&target.root);
        self.targets.borrow_mut().push(target);
    }
    
    pub fn remove_target(&self, target_name: &str) {
        let mut targets = self.targets.borrow_mut();
        if let Some(target) = targets.iter()
            .position(|t| t.name == target_name && t.type_ == self.type_)
            .map(|i| targets.remove(i))
        {
            self.targets_list.remove(&target.root);
        }
    }
    
    pub fn refresh(&self) {
        let targets = if self.type_ == AudioTargetType::App {
            read_config().screen_recorder.audio_app_targets.clone()
        } else {
            read_config().screen_recorder.audio_device_targets.clone()
        };
        
        for target in targets {
            self.add_target(&target);
        }
    }
}
