mod imp {
    use std::cell::{Cell, RefCell};
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;
    use libadwaita::prelude::*;
    
    use crate::utils::gesture;
    use crate::widgets::bar::base::hide_all_expanded_modules;
    
    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::BarModule)]
    pub struct BarModule {
        #[property(get, set = Self::set_minimal_widget, nullable)]
        pub minimal_widget: RefCell<Option<gtk4::Widget>>,
        #[property(get, set = Self::set_expanded_widget, nullable)]
        pub expanded_widget: RefCell<Option<gtk4::Widget>>,
        #[property(get, set = Self::set_expanded)]
        pub expanded: Cell<bool>,
        
        // Animation state
        pub progress: Cell<f64>,
        pub animation: RefCell<Option<libadwaita::Animation>>,
        
        background_widget: RefCell<Option<gtk4::Widget>>,
    }
    
    #[glib::object_subclass]
    impl ObjectSubclass for BarModule {
        const NAME: &'static str = "GrayMeadowsBarModule";
        type Type = super::BarModule;
        type ParentType = gtk4::Widget;
    }
    
    #[glib::derived_properties]
    impl ObjectImpl for BarModule {
        fn constructed(&self) {
            self.parent_constructed();
            
            let obj = self.obj();
            obj.set_valign(gtk4::Align::Start);
            obj.add_css_class("bar-widget");
            
            obj.add_controller(gesture::on_primary_down(clone!(
                #[weak(rename_to = me)] self,
                move |_, x, y| if !me.expanded.get() {
                    // TODO: I should not need to manually determine if the pointer was
                    // actually inside the widget.
                    let mut rectangle = me.obj().allocation();
                    rectangle.set_x(0);
                    rectangle.set_y(0);
                    
                    if rectangle.contains_point(x.max(0.0) as i32, y.max(0.0) as i32) {
                        me.set_expanded(true);
                    }
                }
            )));
    
            obj.add_controller(gesture::on_secondary_down(clone!(
                #[weak(rename_to = me)] self,
                move |_, x, y| if me.expanded.get() {
                    // TODO: I should not need to manually determine if the pointer was
                    // actually inside the widget.
                    let mut rectangle = me.obj().allocation();
                    rectangle.set_x(0);
                    rectangle.set_y(0);
                    
                    if rectangle.contains_point(x.max(0.0) as i32, y.max(0.0) as i32) {
                        me.set_expanded(false);
                    }
                }
            )));
            
            let background = gtk4::Box::builder()
                .vexpand(true)
                .hexpand(true)
                .build();
            background.add_css_class("bar-module-background");
            background.set_parent(&*obj);
            self.background_widget.replace(Some(background.upcast()));
        }
        
        fn dispose(&self) {
            if let Some(child) = self.minimal_widget.borrow_mut().take() {
                child.unparent();
                child.remove_css_class("bar-minimal-widget");
            }
            
            if let Some(child) = self.expanded_widget.borrow_mut().take() {
                child.unparent();
                child.remove_css_class("bar-expanded-widget");
            }
        }
    }
    
    impl WidgetImpl for BarModule {
        fn measure(&self, orientation: gtk4::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            self.minimal_widget.borrow().as_ref().map_or(
                (0, 0, -1, -1),
                |minimal_widget| self.expanded_widget.borrow().as_ref().map_or_else(
                    || minimal_widget.measure(orientation, for_size),
                    |expanded_widget| {
                        let progress = self.progress.get().max(0.0);
                        let minimal_weight = 1.0 - progress;
                        let expanded_weight = progress;
        
                        let (min_min, min_nat, min_min_baseline, min_nat_baseline) = minimal_widget.measure(orientation, for_size);
                        let (exp_min, exp_nat, exp_min_baseline, exp_nat_baseline) = expanded_widget.measure(orientation, for_size);
        
                        let minimum = (min_min as f64)
                            .mul_add(minimal_weight, (exp_min as f64) * expanded_weight)
                            .round() as i32;
                        
                        let natural = (min_nat as f64)
                            .mul_add(minimal_weight, (exp_nat as f64) * expanded_weight)
                            .round() as i32;
        
                        let min_baseline = if min_min_baseline < 0 || exp_min_baseline < 0 {
                            -1
                        } else {
                            (min_min_baseline as f64)
                                .mul_add(minimal_weight, (exp_min_baseline as f64) * expanded_weight)
                                .round() as i32
                        };
        
                        let nat_baseline = if min_nat_baseline < 0 || exp_nat_baseline < 0 {
                            -1
                        } else {
                            (min_nat_baseline as f64)
                                .mul_add(minimal_weight, (exp_nat_baseline as f64) * expanded_weight)
                                .round() as i32
                        };
        
                        (minimum, natural, min_baseline, nat_baseline)
                    }
                )
            )
        }
        
        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            let minimal_nat_height = self.minimal_widget.borrow().as_ref().map_or(height, |minimal_widget| {
                let (_, nat_height, _, _) = minimal_widget.measure(gtk4::Orientation::Vertical, -1);
                let alloc = gtk4::Allocation::new(0, 0, width, nat_height);
                minimal_widget.size_allocate(&alloc, baseline);
                nat_height
            });
    
            let expanded_nat_height = self.expanded_widget.borrow().as_ref().map_or(height, |expanded_widget| {
                let (_, nat_height, _, _) = expanded_widget.measure(gtk4::Orientation::Vertical, -1);
                let alloc = gtk4::Allocation::new(0, 0, width, nat_height);
                expanded_widget.size_allocate(&alloc, baseline);
                nat_height
            });
            
            if let Some(background_widget) = self.background_widget.borrow().as_ref() {
                let progress = self.progress.get();
                let background_height = if progress > 0.0 {
                    ((expanded_nat_height - minimal_nat_height) as f64)
                        .mul_add(progress, minimal_nat_height as f64)
                        .round() as i32
                } else {
                    minimal_nat_height
                };
                
                let alloc = gtk4::Allocation::new(0, 0, width, background_height);
                background_widget.size_allocate(&alloc, baseline);
            }
        }
    
        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            if let Some(background_widget) = self.background_widget.borrow().as_ref() {
                self.obj().snapshot_child(background_widget, snapshot);
            }
            
            if let Some(minimal_widget) = self.minimal_widget.borrow().as_ref() {
                if let Some(expanded_widget) = self.expanded_widget.borrow().as_ref() {
                    let progress = self.progress.get();
                    let width = self.obj().width();
                    let height = self.obj().height();
                    
                    let minimal_nat = self.minimal_widget.borrow().as_ref().map_or((width, height), |minimal_widget| {
                        let (_, nat_width, _, _) = minimal_widget.measure(gtk4::Orientation::Horizontal, -1);
                        let (_, nat_height, _, _) = minimal_widget.measure(gtk4::Orientation::Vertical, -1);
                        let alloc = gtk4::Allocation::new(0, 0, width, nat_height);
                        minimal_widget.size_allocate(&alloc, -1);
                        (nat_width, nat_height)
                    });
            
                    let expanded_nat = self.expanded_widget.borrow().as_ref().map_or((width, height), |expanded_widget| {
                        let (_, nat_width, _, _) = expanded_widget.measure(gtk4::Orientation::Horizontal, -1);
                        let (_, nat_height, _, _) = expanded_widget.measure(gtk4::Orientation::Vertical, -1);
                        let alloc = gtk4::Allocation::new(0, 0, width, nat_height);
                        expanded_widget.size_allocate(&alloc, -1);
                        (nat_width, nat_height)
                    });
                    
                    let expected_width = ((expanded_nat.0 - minimal_nat.0) as f64)
                        .mul_add(progress, minimal_nat.0 as f64)
                        .round() as f32;
                    
                    let expected_height = ((expanded_nat.1 - minimal_nat.1) as f64)
                        .mul_add(progress, minimal_nat.1 as f64)
                        .round() as f32;
    
                    if width > 0 && height > 0 {
                        let progress = progress.clamp(0.0, 1.0);
                        let minimal_opacity = 1.0 - progress;
                        let expanded_opacity = progress;
                        let minimal_blur = progress * super::BLUR_FACTOR_PX as f64;
                        let expanded_blur = (1.0 - progress) * super::BLUR_FACTOR_PX as f64;
    
                        if minimal_opacity > 0.0 || minimal_blur > 0.0 {
                            let minimal_x = (width - minimal_nat.0) as f32 / 2.0;
                            snapshot.save();
                            snapshot.push_clip(&graphene::Rect::new(minimal_x, 0.0, expected_width, expected_height));
                            snapshot.push_opacity(minimal_opacity);
                            snapshot.push_blur(minimal_blur);
                            snapshot.translate(&graphene::Point::new(minimal_x, 0.0));
                            self.obj().snapshot_child(minimal_widget, snapshot);
                            snapshot.pop();
                            snapshot.pop();
                            snapshot.pop();
                            snapshot.restore();
                        }
    
                        if expanded_opacity > 0.0 || expanded_blur > 0.0 {
                            snapshot.push_clip(&graphene::Rect::new(0.0, 0.0, expected_width, expected_height));
                            snapshot.push_opacity(expanded_opacity);
                            snapshot.push_blur(expanded_blur);
                            self.obj().snapshot_child(expanded_widget, snapshot);
                            snapshot.pop();
                            snapshot.pop();
                            snapshot.pop();
                        }
                    }
                } else {
                    self.obj().snapshot_child(minimal_widget, snapshot);
                }
            }
        }
    
        fn unmap(&self) {
            self.parent_unmap();
            if let Some(anim) = self.animation.borrow_mut().take() {
                anim.skip();
            }
            
            self.progress.set(if self.expanded.get() { 1.0 } else { 0.0 });
        }
    }
    
    impl BarModule {
        fn set_minimal_widget(&self, widget: Option<&gtk4::Widget>) {
            let mut stored = self.minimal_widget.borrow_mut();
            if let Some(minimal_widget) = stored.take() {
                minimal_widget.unparent();
                minimal_widget.remove_css_class("bar-minimal-widget");
            }
            
            if let Some(widget) = widget {
                widget.set_parent(&*self.obj());
                widget.set_can_target(!self.expanded.get());
                widget.add_css_class("bar-minimal-widget");
                stored.replace(widget.clone());
            }
        }
        
        fn set_expanded_widget(&self, widget: Option<&gtk4::Widget>) {
            let mut stored = self.expanded_widget.borrow_mut();
            if let Some(expanded_widget) = stored.take() {
                expanded_widget.unparent();
                expanded_widget.remove_css_class("bar-expanded-widget");
            }
            
            if let Some(widget) = widget {
                widget.set_parent(&*self.obj());
                widget.set_can_target(self.expanded.get());
                widget.add_css_class("bar-expanded-widget");
                stored.replace(widget.clone());
            }
        }
        
        fn set_expanded(&self, expanded: bool) {
            if self.expanded_widget.borrow().is_none() || self.expanded.get() == expanded {
                return;
            }
            
            if expanded {
                hide_all_expanded_modules();
            }
            
            if let Some(widget) = self.expanded_widget.borrow().as_ref() {
                widget.set_can_target(expanded);
            }
            
            if let Some(widget) = self.minimal_widget.borrow().as_ref() {
                widget.set_can_target(!expanded);
            }

            self.expanded.set(expanded);
            
            let obj = self.obj();
            let start = self.progress.get();
            let end = if expanded { 1.0 } else { 0.0 };
            
            if let Some(anim) = self.animation.borrow().as_ref() {
                anim.skip();
            }
            
            let target = libadwaita::CallbackAnimationTarget::new(clone!(
               #[weak] obj,
               move |value| {
                   let imp = obj.imp();
                   imp.progress.set(value);
                   obj.queue_resize();
               } 
            ));
            
            let anim = libadwaita::TimedAnimation::builder()
                .widget(&*obj)
                .value_from(start)
                .value_to(end)
                .duration(600)
                .easing(libadwaita::Easing::EaseOutExpo)
                .target(&target)
                .build();
            
            anim.play();
            *self.animation.borrow_mut() = Some(anim.upcast());
        }
    }
}

use crate::APP_LOCAL;

static BLUR_FACTOR_PX: i32 = 32;

glib::wrapper! {
    pub struct BarModule(ObjectSubclass<imp::BarModule>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for BarModule {
    fn default() -> Self {
        glib::Object::builder().build()
    }
}

impl BarModule {
    pub fn with_widgets(minimal_widget: &gtk4::Widget, expanded_widget: &gtk4::Widget) -> Self {
        Self::builder()
            .minimal_widget(minimal_widget)
            .expanded_widget(expanded_widget)
            .build()
    }
    
    pub fn builder() -> BarModuleBuilder {
        BarModuleBuilder::new()
    }
}

pub struct BarModuleBuilder {
    builder: glib::object::ObjectBuilder<'static, BarModule>,
}

impl BarModuleBuilder {
    fn new() -> Self {
        Self {
            builder: glib::Object::builder(),
        }
    }

    pub fn build(self) -> BarModule {
        self.builder.build()
    }

    pub fn minimal_widget(mut self, widget: &gtk4::Widget) -> Self {
        self.builder = self.builder.property("minimal-widget", widget);
        self
    }

    pub fn expanded_widget(mut self, widget: &gtk4::Widget) -> Self {
        self.builder = self.builder.property("expanded-widget", widget);
        self
    }
}

pub fn hide_all_expanded_modules() {
    APP_LOCAL.with(|app| for bar in app.bars.borrow().iter() {
        bar.hide_all_expanded_modules();
    });
}