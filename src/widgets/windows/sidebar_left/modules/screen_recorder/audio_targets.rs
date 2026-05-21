use std::cell::RefCell;
use std::rc::Rc;
use gtk::prelude::*;

use crate::config::read_config;
use crate::ffi::astalwp::WpEvent;
use crate::ffi::astalwp::ffi::EndpointType;
use crate::services::wireplumber;

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

        let target_model = gio::ListStore::new::<glib::BoxedAnyObject>();
        let target_expression = gtk::ClosureExpression::new::<String>(
            &[] as &[gtk::Expression],
            glib::closure!(|obj: glib::Object| {
                let obj = obj
                    .downcast::<glib::BoxedAnyObject>()
                    .expect("DropDown model item must be a BoxedAnyObject");
    
                let borrowed = obj
                    .borrow::<(String, Option<String>)>();

                if let Some(desc) = borrowed.1.as_ref() {
                    format!("{} ({})", borrowed.0, desc)
                } else {
                    borrowed.0.clone()
                }
            }),
        );

        let targets_dropdown = gtk::DropDown::builder()
            .model(&target_model)
            .expression(&target_expression)
            .selected(gtk::INVALID_LIST_POSITION)
            .build();

        // add & remove nodes as we discover them
        glib::spawn_future_local(clone!(
            #[weak] target_model,
            #[weak] targets_dropdown,
            async move {
                let mut receiver = wireplumber::subscribe();
                while let Ok(event) = receiver.recv().await {
                    let remove_from_model = |name: &str| {
                        let mut i = 0;
                        while i < target_model.n_items() {
                            let Some(obj) = target_model.item(i) else {
                                i += 1;
                                continue;
                            };

                            let Ok(any_obj) = obj.downcast::<glib::BoxedAnyObject>() else {
                                i += 1;
                                continue;
                            };

                            let borrowed = any_obj.borrow::<(String, Option<String>)>();
                            if borrowed.0 == name {
                                target_model.remove(i);
                                break;
                            }

                            i += 1;
                        }
                    };

                    match event {
                        WpEvent::CreateStream(node) if type_ == AudioTargetType::App => {
                            target_model.append(&glib::BoxedAnyObject::new((node.description, Some("stream".to_owned()))));
                        }
                        
                        WpEvent::CreateRecorder(node) if type_ == AudioTargetType::App => {
                            target_model.append(&glib::BoxedAnyObject::new((node.description, Some("recorder".to_owned()))));
                        }

                        WpEvent::RemoveStream(node) | WpEvent::RemoveRecorder(node) if type_ == AudioTargetType::App => {
                            remove_from_model(&node.description);
                        }

                        WpEvent::CreateMicrophone(endpoint) | WpEvent::CreateSpeaker(endpoint) if type_ == AudioTargetType::Device => {
                            target_model.append(&glib::BoxedAnyObject::new((endpoint.node.description, match endpoint.type_ {
                                EndpointType::Microphone => Some("microphone".to_owned()),
                                EndpointType::Speaker => Some("speaker".to_owned()),
                                _ => None,
                            })));
                        }

                        WpEvent::RemoveMicrophone(endpoint) | WpEvent::RemoveSpeaker(endpoint) if type_ == AudioTargetType::Device => {
                            remove_from_model(&endpoint.node.description);
                        }

                        _ => {}
                    }

                    // correct the selected property on the dropdown if > items in the model
                    let n_items = target_model.n_items();
                    let selected = targets_dropdown.selected();
                    if n_items == 0 {
                        targets_dropdown.set_selected(gtk::INVALID_LIST_POSITION);
                    } else if selected == gtk::INVALID_LIST_POSITION || selected >= n_items {
                        targets_dropdown.set_selected(0);
                    }
                }
            }
        ));
        
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
        root.append(&targets_dropdown);
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
