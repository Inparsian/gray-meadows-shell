use std::time::Duration;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use gtk4::prelude::*;
use relm4::RelmRemoveAllExt as _;

use crate::singletons::notifications::close_notification_by_id;
use crate::singletons::notifications::wrapper::{Notification, NotificationAction, NotificationCloseReason};
use crate::utils::gesture;
use crate::widgets::common::revealer::{AdwRevealer, AdwRevealerDirection, GEasing};

const NOTIF_TRANSITION_DURATION: u32 = 800; // ms
const DRAG_BEGIN_THRESHOLD: u32 = 30; // px
const DRAG_CONFIRM_THRESHOLD: u32 = 100; // px
const DEFAULT_CSS: &str = ".notification {
    margin-left: 0px;
    margin-right: 0px;
    opacity: 1.0;
    transition: opacity 0.1s ease, margin-left 0.2s ease, margin-right 0.2s ease;
}";

pub enum NotificationDismissAnimation {
    Left,
    Right,
}

impl NotificationDismissAnimation {
    pub fn css(&self, width: i32) -> String {
        let margin_left = if matches!(self, NotificationDismissAnimation::Left) { -width } else { width };
        let margin_right = if matches!(self, NotificationDismissAnimation::Right) { -width } else { width };

        format!(".notification {{
            margin-left: {margin_left}px;
            margin-right: {margin_right}px;
            opacity: 0;
            transition: opacity 0.1s ease, margin-left 0.2s ease, margin-right 0.2s ease;
        }}")
    }
}

#[derive(Clone)]
pub struct NotificationWidget {
    pub notifications_ref: Option<Weak<RefCell<Vec<NotificationWidget>>>>,
    pub notification: Rc<RefCell<Notification>>,
    pub expanded: Rc<RefCell<bool>>,
    pub destroying: Rc<RefCell<bool>>,
    pub bx: gtk4::Box,
    pub root: AdwRevealer,
    pub summary: gtk4::Label,
    pub body: gtk4::Label,
    pub actions_box: gtk4::Box,
    pub style_provider: gtk4::CssProvider,
}

impl glib::clone::Downgrade for NotificationWidget {
    type Weak = NotificationWidgetWeak;

    fn downgrade(&self) -> Self::Weak {
        NotificationWidgetWeak {
            notifications_ref: self.notifications_ref.clone(),
            notification: Rc::downgrade(&self.notification),
            expanded: Rc::downgrade(&self.expanded),
            destroying: Rc::downgrade(&self.destroying),
            bx: glib::clone::Downgrade::downgrade(&self.bx),
            root: glib::clone::Downgrade::downgrade(&self.root),
            summary: glib::clone::Downgrade::downgrade(&self.summary),
            body: glib::clone::Downgrade::downgrade(&self.body),
            actions_box: glib::clone::Downgrade::downgrade(&self.actions_box),
            style_provider: glib::clone::Downgrade::downgrade(&self.style_provider),
        }
    }
}

pub struct NotificationWidgetWeak {
    pub notifications_ref: Option<Weak<RefCell<Vec<NotificationWidget>>>>,
    pub notification: Weak<RefCell<Notification>>,
    pub expanded: Weak<RefCell<bool>>,
    pub destroying: Weak<RefCell<bool>>,
    pub bx: glib::WeakRef<gtk4::Box>,
    pub root: glib::WeakRef<AdwRevealer>,
    pub summary: glib::WeakRef<gtk4::Label>,
    pub body: glib::WeakRef<gtk4::Label>,
    pub actions_box: glib::WeakRef<gtk4::Box>,
    pub style_provider: glib::WeakRef<gtk4::CssProvider>,
}

impl glib::clone::Upgrade for NotificationWidgetWeak {
    type Strong = NotificationWidget;

    fn upgrade(&self) -> Option<Self::Strong> {
        Some(NotificationWidget {
            notifications_ref: self.notifications_ref.clone(),
            notification: self.notification.upgrade()?,
            expanded: self.expanded.upgrade()?,
            destroying: self.destroying.upgrade()?,
            bx: self.bx.upgrade()?,
            root: self.root.upgrade()?,
            summary: self.summary.upgrade()?,
            body: self.body.upgrade()?,
            actions_box: self.actions_box.upgrade()?,
            style_provider: self.style_provider.upgrade()?,
        })
    }
}

impl NotificationWidget {
    fn make_action_button(
        notification: &Notification,
        action: &NotificationAction,
    ) -> gtk4::Button {
        let button = gtk4::Button::with_label(&action.localized_name);
        button.set_css_classes(&["notification-action-button"]);
        button.connect_clicked({
            let notification_id = notification.id;
            let action_id = action.id.clone();
            move |_| {
                crate::singletons::notifications::invoke_notification_action(notification_id, &action_id);
            }
        });
        
        button
    }

    pub fn new(notification: Notification) -> Self {
        let style_provider = gtk4::CssProvider::new();
        let drag_gesture = gtk4::GestureDrag::new();

        view! {
            summary = gtk4::Label {
                set_label: &notification.summary,
                set_css_classes: &["notification-summary"],
                set_xalign: 0.0,
                set_hexpand: true,
                set_ellipsize: gtk4::pango::EllipsizeMode::End,
            },

            body = gtk4::Label {
                set_label: &notification.body,
                set_css_classes: &["notification-body"],
                set_xalign: 0.0,
                set_hexpand: true,
                set_ellipsize: gtk4::pango::EllipsizeMode::End,
                set_wrap_mode: gtk4::pango::WrapMode::WordChar,
                set_wrap: false,
            },

            actions_box = gtk4::Box {
                set_css_classes: &["notification-actions"],
                set_orientation: gtk4::Orientation::Horizontal,
                set_homogeneous: true,
                set_spacing: 4,
            },

            actions = gtk4::Revealer {
                set_reveal_child: false,
                set_transition_type: gtk4::RevealerTransitionType::SlideDown,
                set_transition_duration: 150,
                set_child: Some(&actions_box),
            },

            bx = gtk4::Box {
                set_css_classes: &["notification"],
                set_orientation: gtk4::Orientation::Vertical,
                set_spacing: 0,
                set_hexpand: true,

                gtk4::Box {
                    set_css_classes: &["notification-content"],
                    set_orientation: gtk4::Orientation::Vertical,
                    set_spacing: 4,

                    gtk4::Box {
                        set_css_classes: &["notification-header"],
                        set_orientation: gtk4::Orientation::Horizontal,
                        set_spacing: 4,

                        append: &summary,

                        gtk4::Label {
                            set_css_classes: &["notification-timestamp"],
                            set_label: &chrono::Local::now().format("%I:%M %p").to_string(),
                            set_xalign: 1.0,
                            set_halign: gtk4::Align::End,
                            set_hexpand: true,
                        }
                    },
                    append: &body,
                },

                append: &actions,
            },

            root = AdwRevealer {
                set_reveal: false,
                set_transition_direction: AdwRevealerDirection::Down,
                set_transition_duration: NOTIF_TRANSITION_DURATION,
                set_show_easing: GEasing::EaseOutBounce,
                set_hide_easing: GEasing::EaseOutExpo,
                set_hexpand: true,
                set_child: Some(&bx.clone().upcast()),
                set_overflow: gtk4::Overflow::Visible,
            }
        }

        for action in &notification.actions {
            let button = Self::make_action_button(&notification, action);
            actions_box.append(&button);
        }

        let me = NotificationWidget {
            notifications_ref: None,
            notification: Rc::new(RefCell::new(notification)),
            expanded: Rc::new(RefCell::new(false)),
            destroying: Rc::new(RefCell::new(false)),
            bx,
            root,
            summary,
            body,
            actions_box,
            style_provider,
        };

        drag_gesture.connect_drag_update(clone!(
            #[weak] me,
            move |_, offset_x, _| {
                if *me.destroying.borrow() {
                    return;
                }
                
                if offset_x.abs() as u32 >= DRAG_BEGIN_THRESHOLD {
                    let margin = me.get_offset_margin(offset_x);
                    let opacity = me.get_offset_opacity(offset_x);
                    me.style_provider.load_from_data(&format!(
                        ".notification {{
                        margin-left: {}px;
                        margin-right: {}px; 
                        opacity: {:.2};
                        }}",
                        margin,
                        -margin,
                        opacity,
                    ));
                } else {
                    me.style_provider.load_from_data(DEFAULT_CSS);
                }
            }
        ));

        drag_gesture.connect_drag_end(clone!(
            #[weak] me,
            move |_, offset_x, _| {
                if *me.destroying.borrow() {
                    return;
                }
                
                if offset_x.abs() as u32 >= DRAG_CONFIRM_THRESHOLD {
                    let animation = if offset_x < 0.0 {
                        NotificationDismissAnimation::Left
                    } else {
                        NotificationDismissAnimation::Right
                    };

                    me.destroy(Some(animation));
                    let _ = close_notification_by_id(
                        me.notification.borrow().id,
                        NotificationCloseReason::Dismissed
                    );
                } else {
                    me.style_provider.load_from_data(DEFAULT_CSS);
                }
            }
        ));

        me.root.add_controller(drag_gesture);
        me.root.add_controller(gesture::on_enter(clone!(
            #[weak] me,
            #[weak] actions,
            move |_, _| {
                actions.set_reveal_child(!me.notification.borrow().actions.is_empty());
                me.set_expand_state(true);
            }
        )));
        me.root.add_controller(gesture::on_leave(clone!(
            #[weak] me,
            move || {
                actions.set_reveal_child(false);
                me.set_expand_state(false);
            }
        )));

        me.bx.style_context().add_provider(&me.style_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

        me.root.connect_map(move |revealer| {
            revealer.set_reveal(true);
        });

        me
    }
    
    pub fn get_removal_operation_widgets(&self) -> (Option<gtk4::Widget>, Option<gtk4::Box>) {
        let mut current_child = self.root.clone().upcast::<gtk4::Widget>();
        
        let mut recursion_depth = 0;
        while let Some(parent) = current_child.parent() {
            // We should ideally only go at most 2 levels up. If we go higher than this,
            // flattening the widget structure should be considered.
            if recursion_depth >= 2 {
                break;
            }
            
            if let Some(bx) = parent.downcast_ref::<gtk4::Box>() {
                return (Some(current_child), Some(bx.clone()));
            }
            
            current_child = parent;
            recursion_depth += 1;
        }
        
        warn!("Could not find widget to remove from notifications list");
        (None, None)
    }

    pub fn set_notifications_ref(&mut self, notifications_ref: &Rc<RefCell<Vec<NotificationWidget>>>) {
        self.notifications_ref = Some(Rc::downgrade(notifications_ref));
    }

    pub fn set_expand_state(&self, expanded: bool) {
        self.expanded.replace(expanded);
        if !expanded && !*self.destroying.borrow() {
            self.body.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            self.body.set_wrap(false);
        } else {
            self.body.set_ellipsize(gtk4::pango::EllipsizeMode::None);
            self.body.set_wrap(true);
        }
    }

    pub fn update(&self, notification: &Notification) {
        self.notification.replace(notification.clone());
        self.summary.set_label(&notification.summary);
        self.body.set_label(&notification.body);

        self.actions_box.remove_all();
        for action in &notification.actions {
            let button = Self::make_action_button(notification, action);
            self.actions_box.append(&button);
        }
    }

    pub fn get_offset_margin(&self, offset_x: f64) -> i32 {
        let allocation = self.bx.allocation();
        let width = allocation.width() as f64;
        (offset_x / width * width).clamp(-width, width) as i32
    }

    pub fn get_offset_opacity(&self, offset_x: f64) -> f64 {
        let allocation = self.bx.allocation();
        let width = allocation.width() as f64;
        let progress = (offset_x.abs() / width).clamp(0.0, 1.0);
        1.0 - progress * progress
    }

    pub async fn wait_until_not_expanded(&self) {
        while *self.expanded.borrow() {
            glib::timeout_future(Duration::from_millis(50)).await;
        }
    }

    pub fn queue_destroy(&self, animation: Option<NotificationDismissAnimation>) {
        // Wait until not expanded. if expanded, wait again
        glib::spawn_future_local(clone!(
            #[weak(rename_to = me)] self,
            async move {
                me.wait_until_not_expanded().await;

                glib::timeout_add_local_once(Duration::from_millis(1000), clone!(
                    #[weak] me,
                    move || if *me.expanded.borrow() {
                        me.queue_destroy(animation);
                    } else {
                        me.destroy(animation);
                    }
                ));
            }
        ));
    }

    pub fn destroy(&self, animation: Option<NotificationDismissAnimation>) {
        if *self.destroying.borrow() {
            return;
        }

        self.destroying.replace(true);
        self.set_expand_state(false);
        self.root.set_reveal(false);

        if let Some(anim) = animation {
            let allocation = self.bx.allocation();
            let width = allocation.width();
            self.style_provider.load_from_data(&anim.css(width));
        }

        if let Some(notifications) = self.notifications_ref.as_ref().and_then(|r| r.upgrade()) {
            notifications.borrow_mut().retain(|w| !w.root.eq(&self.root));
        }

        glib::timeout_add_local_once(
            Duration::from_millis(NOTIF_TRANSITION_DURATION as u64),
            clone!(
                #[strong(rename_to = me)] self,
                move || if let (Some(child), Some(parent)) = me.get_removal_operation_widgets() {
                    parent.remove(&child);
                }
            )
        );
    }
}