use std::time::Duration;
use gtk4::prelude::*;

use crate::singletons::notifications::wrapper::Notification;

const NOTIF_TRANSITION_DURATION: u32 = 175; // ms
#[allow(dead_code)]
const DISMISS_ANIMATION_DELAY: u32 = 75; // ms
const DRAG_BEGIN_THRESHOLD: u32 = 30; // px
const DRAG_CONFIRM_THRESHOLD: u32 = 100; // px

fn left_animation_css(width: i32) -> String {
    format!(
        ".notification {{
            margin-right: {}px;
            margin-left: {}px;
            opacity: 0;
            transition: opacity 0.2s ease, margin-left 0.2s ease, margin-right 0.2s ease;
        }}",
        width,
        -width
    )
}

fn right_animation_css(width: i32) -> String {
    format!(
        ".notification {{
            margin-left: {}px;
            margin-right: {}px;
            opacity: 0;
            transition: opacity 0.2s ease, margin-left 0.2s ease, margin-right 0.2s ease;
        }}",
        width,
        -width
    )
}

#[derive(Clone)]
pub struct NotificationWidget {
    #[allow(dead_code)]
    pub notification: Notification,
    pub bx: gtk4::Box,
    pub root: gtk4::Revealer,
    pub style_provider: gtk4::CssProvider,
}

impl NotificationWidget {
    pub fn new(notification: Notification) -> Self {
        let style_provider = gtk4::CssProvider::new();
        let drag_gesture = gtk4::GestureDrag::new();

        view! {
            bx = gtk4::Box {
                set_css_classes: &["notification"],
                set_orientation: gtk4::Orientation::Vertical,
                set_spacing: 0,
                set_hexpand: true,

                gtk4::Box {
                    set_css_classes: &["notification-content"],
                    set_orientation: gtk4::Orientation::Vertical,
                    set_spacing: 4,

                    gtk4::Label {
                        set_label: &notification.summary,
                        set_css_classes: &["notification-summary"],
                        set_xalign: 0.0,
                        set_hexpand: true,
                        set_ellipsize: gtk4::pango::EllipsizeMode::End,
                    },

                    gtk4::Label {
                        set_label: &notification.body,
                        set_css_classes: &["notification-body"],
                        set_xalign: 0.0,
                        set_hexpand: true,
                        set_ellipsize: gtk4::pango::EllipsizeMode::End,
                    },
                },
            },

            root = gtk4::Revealer {
                set_reveal_child: false,
                set_transition_type: gtk4::RevealerTransitionType::SlideDown,
                set_transition_duration: NOTIF_TRANSITION_DURATION,
                set_hexpand: true,
                set_child: Some(&bx),
                set_overflow: gtk4::Overflow::Visible,
            }
        }

        let me = NotificationWidget {
            notification,
            bx,
            root,
            style_provider,
        };

        drag_gesture.connect_drag_update({
            let style_provider = me.style_provider.clone();
            let me = me.clone();
            move |_, offset_x, _| {
                if offset_x.abs() as u32 >= DRAG_BEGIN_THRESHOLD {
                    let margin = me.get_offset_margin(offset_x);
                    let opacity = me.get_offset_opacity(offset_x);
                    style_provider.load_from_data(&format!(
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
                    style_provider.load_from_data(
                        ".notification { margin-left: 0px; margin-right: 0px; transition: margin-left 0.2s ease, margin-right 0.2s ease; }",
                    );
                }
            }
        });

        drag_gesture.connect_drag_end({
            let me = me.clone();
            let style_provider = me.style_provider.clone();
            move |_, offset_x, _| {
                if offset_x.abs() as u32 >= DRAG_CONFIRM_THRESHOLD {
                    let allocation = me.bx.allocation();
                    let width = allocation.width();
                    if offset_x < 0.0 {
                        style_provider.load_from_data(&left_animation_css(width));
                    } else {
                        style_provider.load_from_data(&right_animation_css(width));
                    }
                    me.destroy();
                } else {
                    style_provider.load_from_data(
                        ".notification { margin-left: 0px; margin-right: 0px; transition: margin-left 0.2s ease, margin-right 0.2s ease; }",
                    );
                }
            }
        });

        me.root.add_controller(drag_gesture);
        me.bx.style_context().add_provider(&me.style_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

        me.root.connect_map(move |revealer| {
            revealer.set_reveal_child(true);
        });

        me
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

    pub fn destroy(&self) {
        self.root.set_reveal_child(false);

        gtk4::glib::timeout_add_local_once(
            Duration::from_millis(NOTIF_TRANSITION_DURATION as u64),
            {
                let me = self.clone();
                move || if let Some(parent) = me.root.parent()
                    && let Some(bx) = parent.downcast_ref::<gtk4::Box>()
                {
                    bx.remove(&me.root);
                }
            }
        );
    }
}