mod imp {
    use std::cell::{Cell, RefCell};
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;
    use libadwaita::prelude::*;
    
    use super::{GEasing, AdwRevealerDirection};
    
    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::AdwRevealer)]
    pub struct AdwRevealer {
        #[property(get, set = Self::set_child, nullable)]
        pub child: RefCell<Option<gtk4::Widget>>,
        #[property(get, set = Self::set_reveal)]
        pub reveal: Cell<bool>,
        #[property(get, set)]
        pub transition_duration: Cell<u32>,
        #[property(get, set, default)]
        pub transition_direction: Cell<AdwRevealerDirection>,
        #[property(get, set, default)]
        pub show_easing: Cell<GEasing>,
        #[property(get, set, default)]
        pub hide_easing: Cell<GEasing>,

        // Animation state
        pub progress: Cell<f64>,
        pub animation: RefCell<Option<libadwaita::Animation>>,
    }
    
    #[glib::object_subclass]
    impl ObjectSubclass for AdwRevealer {
        const NAME: &'static str = "AdwRevealer";
        type Type = super::AdwRevealer;
        type ParentType = gtk4::Widget;
    }
    
    #[glib::derived_properties]
    impl ObjectImpl for AdwRevealer {
        fn constructed(&self) {
            self.parent_constructed();
        }
        
        fn dispose(&self) {
            if let Some(child) = self.child.borrow_mut().take() {
                child.unparent();
            }
        }
    }
    
    impl WidgetImpl for AdwRevealer {
        fn measure(&self, orientation: gtk4::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            if let Some(child) = self.child.borrow().as_ref() && child.should_layout() {
                fn get_scale(orientation: gtk4::Orientation, direction: AdwRevealerDirection, progress: f32) -> f32 {
                    match (orientation, direction) {
                        (gtk4::Orientation::Horizontal, AdwRevealerDirection::Right | AdwRevealerDirection::Left)
                        | (gtk4::Orientation::Vertical, AdwRevealerDirection::Down | AdwRevealerDirection::Up) => progress,
                        _ => 1.0,
                    }
                }

                let opposite_orientation = if orientation == gtk4::Orientation::Horizontal {
                    gtk4::Orientation::Vertical
                } else {
                    gtk4::Orientation::Horizontal
                };

                let progress = self.progress.get() as f32;
                let direction = self.transition_direction.get();
                let scale = get_scale(orientation, direction, progress);
                let opposite_scale = get_scale(opposite_orientation, direction, progress);

                let (min, nat, min_baseline, nat_baseline) = child.measure(orientation, if opposite_scale == 0.0 {
                    -1
                } else if for_size >= 0 {
                    (for_size as f32 / opposite_scale).ceil() as i32
                } else {
                    for_size
                });

                let (min, nat) = if min < 0 || nat < 0 {
                    (min, nat)
                } else {
                    (
                        (min as f32 * scale).ceil() as i32,
                        (nat as f32 * scale).ceil() as i32,
                    )
                };

                (min, nat, min_baseline, nat_baseline)
            } else {
                (0, 0, -1, -1)
            }
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            if let Some(child) = self.child.borrow().as_ref() && child.should_layout() {
                let progress = self.progress.get();
                let direction = self.transition_direction.get();

                // width & height are intentionally swapped, for_size is for the *opposite* orientation
                let child_width = match direction {
                    AdwRevealerDirection::Right | AdwRevealerDirection::Left => child.measure(gtk4::Orientation::Horizontal, height).1,
                    _ => width,
                };

                let child_height = match direction {
                    AdwRevealerDirection::Down | AdwRevealerDirection::Up => child.measure(gtk4::Orientation::Vertical, width).1,
                    _ => height,
                };

                let transform = match direction {
                    AdwRevealerDirection::Right => Some(gsk4::Transform::new().translate(&graphene::Point::new((progress as f32 - 1.0) * child_width as f32, 0.0))),
                    AdwRevealerDirection::Down => Some(gsk4::Transform::new().translate(&graphene::Point::new(0.0, (progress as f32 - 1.0) * child_height as f32))),
                    _ => None,
                };

                child.allocate(child_width, child_height, baseline, transform);
            }
        }

        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            if let Some(child) = self.child.borrow().as_ref() && child.should_layout() {
                let obj = self.obj();
                let width = obj.width() as f32;
                let height = obj.height() as f32;

                if width > 0.0 && height > 0.0 {
                    if obj.overflow() == gtk4::Overflow::Hidden {
                        snapshot.push_clip(&graphene::Rect::new(0.0, 0.0, width, height));
                    }
                    obj.snapshot_child(child, snapshot);
                    if obj.overflow() == gtk4::Overflow::Hidden {
                        snapshot.pop();
                    }
                }
            }
        }
        
        fn compute_expand(&self, hexpand: &mut bool, vexpand: &mut bool) {
            if let Some(child) = self.child.borrow().as_ref() {
                *hexpand = child.compute_expand(gtk4::Orientation::Horizontal);
                *vexpand = child.compute_expand(gtk4::Orientation::Vertical);
            } else {
                *hexpand = false;
                *vexpand = false;
            }
        }
        
        fn request_mode(&self) -> gtk4::SizeRequestMode {
            self.child.borrow().as_ref().map_or(gtk4::SizeRequestMode::ConstantSize, |child| child.request_mode())
        }
        
        fn unmap(&self) {
            self.parent_unmap();
            if let Some(anim) = self.animation.borrow_mut().take() {
                anim.skip();
            }
            
            self.progress.set(if self.reveal.get() { 1.0 } else { 0.0 });
        }
    }
    
    impl AdwRevealer {
        fn set_child(&self, widget: Option<&gtk4::Widget>) {
            let mut stored = self.child.borrow_mut();
            if let Some(child) = stored.take() {
                child.unparent();
            }
            
            if let Some(widget) = widget {
                widget.set_parent(&*self.obj());
                stored.replace(widget.clone());
            }
        }
        
        fn set_reveal(&self, reveal: bool) {
            if self.reveal.get() == reveal {
                return;
            }
            
            self.reveal.set(reveal);
            
            let obj = self.obj();
            let start = self.progress.get();
            let end = if reveal { 1.0 } else { 0.0 };
            
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
            
            let duration = obj.transition_duration();
            let show_reveal = obj.show_easing().to_adw();
            let hide_reveal = obj.hide_easing().to_adw();
            let anim = libadwaita::TimedAnimation::builder()
                .widget(&*obj)
                .value_from(start)
                .value_to(end)
                .duration(duration)
                .easing(if reveal { show_reveal } else { hide_reveal })
                .target(&target)
                .build();
            
            anim.play();
            *self.animation.borrow_mut() = Some(anim.upcast());
        }
    }
}

use gtk4::prelude::IsA;

#[derive(Debug, Copy, Clone, PartialEq, Eq, glib::Enum, Default)]
#[enum_type(name = "GEasing")]
/// Copied from libadwaita to implement glib::Enum
pub enum GEasing {
    #[default]
    Linear,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl GEasing {
    fn to_adw(self) -> libadwaita::Easing {
        match self {
            Self::Linear => libadwaita::Easing::Linear,
            Self::EaseInQuad => libadwaita::Easing::EaseInQuad,
            Self::EaseOutQuad => libadwaita::Easing::EaseOutQuad,
            Self::EaseInOutQuad => libadwaita::Easing::EaseInOutQuad,
            Self::EaseInCubic => libadwaita::Easing::EaseInCubic,
            Self::EaseOutCubic => libadwaita::Easing::EaseOutCubic,
            Self::EaseInOutCubic => libadwaita::Easing::EaseInOutCubic,
            Self::EaseInQuart => libadwaita::Easing::EaseInQuart,
            Self::EaseOutQuart => libadwaita::Easing::EaseOutQuart,
            Self::EaseInOutQuart => libadwaita::Easing::EaseInOutQuart,
            Self::EaseInQuint => libadwaita::Easing::EaseInQuint,
            Self::EaseOutQuint => libadwaita::Easing::EaseOutQuint,
            Self::EaseInOutQuint => libadwaita::Easing::EaseInOutQuint,
            Self::EaseInSine => libadwaita::Easing::EaseInSine,
            Self::EaseOutSine => libadwaita::Easing::EaseOutSine,
            Self::EaseInOutSine => libadwaita::Easing::EaseInOutSine,
            Self::EaseInExpo => libadwaita::Easing::EaseInExpo,
            Self::EaseOutExpo => libadwaita::Easing::EaseOutExpo,
            Self::EaseInOutExpo => libadwaita::Easing::EaseInOutExpo,
            Self::EaseInCirc => libadwaita::Easing::EaseInCirc,
            Self::EaseOutCirc => libadwaita::Easing::EaseOutCirc,
            Self::EaseInOutCirc => libadwaita::Easing::EaseInOutCirc,
            Self::EaseInElastic => libadwaita::Easing::EaseInElastic,
            Self::EaseOutElastic => libadwaita::Easing::EaseOutElastic,
            Self::EaseInOutElastic => libadwaita::Easing::EaseInOutElastic,
            Self::EaseInBack => libadwaita::Easing::EaseInBack,
            Self::EaseOutBack => libadwaita::Easing::EaseOutBack,
            Self::EaseInOutBack => libadwaita::Easing::EaseInOutBack,
            Self::EaseInBounce => libadwaita::Easing::EaseInBounce,
            Self::EaseOutBounce => libadwaita::Easing::EaseOutBounce,
            Self::EaseInOutBounce => libadwaita::Easing::EaseInOutBounce,
            Self::Ease => libadwaita::Easing::Ease,
            Self::EaseIn => libadwaita::Easing::EaseIn,
            Self::EaseOut => libadwaita::Easing::EaseOut,
            Self::EaseInOut => libadwaita::Easing::EaseInOut,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, glib::Enum, Default)]
#[enum_type(name = "GmRevealerDirection")]
// The direction of the revealer
pub enum AdwRevealerDirection {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

glib::wrapper! {
    pub struct AdwRevealer(ObjectSubclass<imp::AdwRevealer>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for AdwRevealer {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl AdwRevealer {
    /// Helper function to avoid having to clone and/or upcast Widgets.
    pub fn set_child_from<W: IsA<gtk4::Widget>>(&self, child: Option<&W>) {
        let child: Option<&gtk4::Widget> = child.map(|w| w.as_ref());
        self.set_child(child);
    }
    
    pub fn builder() -> AdwRevealerBuilder {
        AdwRevealerBuilder::new()
    }
}

pub struct AdwRevealerBuilder {
    builder: glib::object::ObjectBuilder<'static, AdwRevealer>,
}

impl AdwRevealerBuilder {
    fn new() -> Self {
        Self {
            builder: glib::Object::builder()
                .property("overflow", gtk4::Overflow::Hidden),
        }
    }

    pub fn child<W: IsA<gtk4::Widget>>(mut self, child: Option<&W>) -> Self {
        let child: Option<&gtk4::Widget> = child.map(|w| w.as_ref());
        self.builder = self.builder.property("child", child);
        self
    }
    
    pub fn css_classes(mut self, classes: impl Into<glib::StrV>) -> Self {
        self.builder = self.builder.property("css-classes", classes.into());
        self
    }
    
    pub fn overflow(mut self, overflow: gtk4::Overflow) -> Self {
        self.builder = self.builder.property("overflow", overflow);
        self
    }

    pub fn reveal(mut self, reveal: bool) -> Self {
        self.builder = self.builder.property("reveal", reveal);
        self
    }

    pub fn transition_duration(mut self, transition_duration: u32) -> Self {
        self.builder = self.builder.property("transition-duration", transition_duration);
        self
    }

    pub fn transition_direction(mut self, transition_direction: AdwRevealerDirection) -> Self {
        self.builder = self.builder.property("transition-direction", transition_direction);
        self
    }

    pub fn show_easing(mut self, show_easing: GEasing) -> Self {
        self.builder = self.builder.property("show-easing", show_easing);
        self
    }

    pub fn hide_easing(mut self, hide_easing: GEasing) -> Self {
        self.builder = self.builder.property("hide-easing", hide_easing);
        self
    }

    pub fn build(self) -> AdwRevealer {
        self.builder.build()
    }
}