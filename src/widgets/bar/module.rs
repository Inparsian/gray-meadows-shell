use std::{cell::RefCell, rc::Rc, time::Duration};
use gtk4::prelude::*;

use crate::APP_LOCAL;
use crate::utils::gesture;
use crate::utils::timeout::Timeout;
use crate::utils::allocation_watcher::{AllocationWatcher, AllocationWatcherOptions};

static TRANSITION_DURATION: f64 = 0.4;
static DOWNSCALE_FACTOR: f64 = 0.000_000_001;
static BLUR_FACTOR_PX: i32 = 8;

#[derive(Clone)]
pub struct BarModule {
    timeout: Timeout,
    pub minimal: gtk4::Widget,
    pub expanded: gtk4::Widget,
    pub minimal_provider: gtk4::CssProvider,
    pub expanded_provider: gtk4::CssProvider,
    pub is_expanded: Rc<RefCell<bool>>
}

impl BarModule {
    pub fn new(
        minimal: impl IsA<gtk4::Widget>,
        expanded: impl IsA<gtk4::Widget>
    ) -> Self {
        let minimal_provider = gtk4::CssProvider::new();
        let expanded_provider = gtk4::CssProvider::new();

        minimal.style_context().add_provider(&minimal_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
        expanded.style_context().add_provider(&expanded_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
        minimal.style_context().add_class("bar-minimal-wrapper");
        expanded.style_context().add_class("bar-expanded-wrapper");
        minimal.set_halign(gtk4::Align::Center);
        expanded.set_halign(gtk4::Align::Center);

        let module = BarModule {
            timeout: Timeout::default(),
            minimal: minimal.upcast(),
            expanded: expanded.upcast(),
            minimal_provider,
            expanded_provider,
            is_expanded: Rc::new(RefCell::new(false))
        };

        let expanded_watcher = AllocationWatcher::new(&module.expanded, AllocationWatcherOptions {
            timeout_millis: 5000,
            max_allocation_width: Some(600),
            max_allocation_height: None,
            min_allocation_width: 64,
            min_allocation_height: 32,
        });

        // Hide expanded after we get it's allocation and set the style for it
        // TODO: I should seriously subclass bar modules instead of using this quick hack
        expanded_watcher.one_shot_future({
            let expanded = module.expanded.clone();
            let expanded_provider = module.expanded_provider.clone();
            let last_received_allocation = expanded_watcher.last_received_allocation.clone();
            async move {
                if let Some((h_middle, v_middle)) = last_received_allocation.get()
                    .map(|alloc| (alloc.width() / 2, alloc.height() / 2))
                {
                    expanded_provider.load_from_data(&format!(
                        ".bar-expanded-wrapper {{
                            opacity: 0;
                            transform: scale({DOWNSCALE_FACTOR});
                            filter: blur({BLUR_FACTOR_PX}px);
                            margin: -{v_middle}px -{h_middle}px -{v_middle}px -{h_middle}px;
                        }}",
                    ));

                    expanded.set_visible(false);
                }
            }
        });

        module
    }

    pub fn is_expanded(&self) -> bool {
        *self.is_expanded.borrow()
    }

    pub fn set_expanded(&self, expanded: bool) {
        if expanded == self.is_expanded() {
            return;
        }

        // collapse all other modules if this one is expanding
        if expanded {
            hide_all_expanded_modules();
        }

        *self.is_expanded.borrow_mut() = expanded;
        self.animate_fade_slide_down(expanded);
    }

    pub fn toggle_expanded(&self) {
        let expanding = !self.is_expanded();

        // collapse all other modules if this one is expanding
        if expanding {
            hide_all_expanded_modules();
        }

        *self.is_expanded.borrow_mut() = expanding;
        self.animate_fade_slide_down(expanding);
    }

    fn animate_fade_slide_down(&self, expanding: bool) {
        if expanding {
            self.expanded.set_visible(true);
        } else {
            self.minimal.set_visible(true);
        }

        let minimal_bounds = self.minimal.compute_bounds(&self.minimal).unwrap();
        let expanded_bounds = self.expanded.compute_bounds(&self.expanded).unwrap();
        let minimal_height = minimal_bounds.height() as f64;
        let expanded_width = expanded_bounds.width() as f64;
        let expanded_height = expanded_bounds.height() as f64;
        
        if expanding {
            self.minimal.set_sensitive(false);
            self.expanded.set_sensitive(true);
            self.minimal.add_css_class("expanding");
            self.expanded.add_css_class("expanding");
            self.minimal_provider.load_from_data(&format!(
                ".bar-minimal-wrapper {{
                    opacity: 0;
                    transform: scale({DOWNSCALE_FACTOR});
                    filter: blur({BLUR_FACTOR_PX}px);
                    margin: -{}px 0px -{}px 0px;
                    transition-duration: {TRANSITION_DURATION}s;
                }}",
                minimal_height/2.0, minimal_height/2.0,
            ));
            self.expanded_provider.load_from_data(&format!(
                ".bar-expanded-wrapper {{
                    opacity: 1;
                    transform: scale(1.0);
                    filter: blur(0px);
                    margin: 0px 0px 0px 0px;
                    transition-duration: {TRANSITION_DURATION}s;
                }}"
            ));
        } else {
            self.minimal.set_sensitive(true);
            self.expanded.set_sensitive(false);
            self.minimal.add_css_class("collapsing");
            self.expanded.add_css_class("collapsing");
            self.expanded_provider.load_from_data(&format!(
                ".bar-expanded-wrapper {{
                    opacity: 0;
                    transform: scale({DOWNSCALE_FACTOR});
                    filter: blur({BLUR_FACTOR_PX}px);
                    margin: -{}px -{}px -{}px -{}px;
                    transition-duration: {TRANSITION_DURATION}s;
                }}",
                expanded_height/2.0, expanded_width/2.0, expanded_height/2.0, expanded_width/2.0
            ));
            self.minimal_provider.load_from_data(&format!(
                ".bar-minimal-wrapper {{
                    opacity: 1;
                    transform: scale(1.0);
                    filter: blur(0px);
                    margin: 0px 0px 0px 0px;
                    transition-duration: {TRANSITION_DURATION}s;
                }}"
            ));
        }

        self.timeout.set(Duration::from_secs_f64(TRANSITION_DURATION), {
            let minimal = self.minimal.clone();
            let expanded = self.expanded.clone();
            let is_expanded = self.is_expanded.clone();
            move || {
                if expanding && is_expanded.borrow().to_owned() {
                    minimal.set_visible(false);
                } else if !is_expanded.borrow().to_owned() {
                    expanded.set_visible(false);
                }

                minimal.remove_css_class("collapsing");
                minimal.remove_css_class("expanding");
                expanded.remove_css_class("collapsing");
                expanded.remove_css_class("expanding");
            }
        });
    }
}

#[derive(Clone)]
pub struct BarModuleWrapper {
    pub bx: gtk4::Box,
    pub module: BarModule,
}

impl BarModuleWrapper {
    pub fn new(module: BarModule, classes: &[&str]) -> Self {
        let mut css_classes = vec!["bar-widget"];
        css_classes.extend_from_slice(classes);
        
        let widget_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        widget_box.set_css_classes(&css_classes);
        widget_box.append(&module.minimal);
        widget_box.append(&module.expanded);

        let wrapper_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        wrapper_box.set_css_classes(&["bar-widget-wrapper"]);
        wrapper_box.set_hexpand(false);
        wrapper_box.set_valign(gtk4::Align::Start);
        wrapper_box.append(&widget_box);

        wrapper_box.add_controller(gesture::on_primary_down({
            let module = module.clone();
            move |_, _, _| if !module.is_expanded() {
                module.set_expanded(true);
            }
        }));

        wrapper_box.add_controller(gesture::on_secondary_down({
            let module = module.clone();
            move |_, _, _| if module.is_expanded() {
                module.set_expanded(false);
            }
        }));

        Self {
            bx: wrapper_box,
            module
        }
    }
}

pub fn hide_all_expanded_modules() {
    APP_LOCAL.with(|app| {
        for bar in app.bars.borrow().iter() {
            bar.hide_all_expanded_modules();
        }
    });
}