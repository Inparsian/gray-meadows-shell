use std::{cell::RefCell, rc::Rc};
use gtk4::prelude::*;

static TRANSITION_DURATION: f64 = 0.4;
static DOWNSCALE_FACTOR: f64 = 0.000_000_001;
static BLUR_FACTOR_PX: i32 = 8;

#[derive(Clone)]
pub struct BarModule {
    tx: tokio::sync::broadcast::Sender<(i32, i32)>,
    pub minimal: gtk4::Widget,
    pub expanded: gtk4::Widget,
    pub minimal_provider: gtk4::CssProvider,
    pub expanded_provider: gtk4::CssProvider,
    pub is_expanded: Rc<RefCell<bool>>
}

impl BarModule {
    pub fn new(minimal: gtk4::Widget, expanded: gtk4::Widget) -> Self {
        let minimal_provider = gtk4::CssProvider::new();
        let expanded_provider = gtk4::CssProvider::new();

        minimal.style_context().add_provider(&minimal_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
        expanded.style_context().add_provider(&expanded_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
        minimal.style_context().add_class("bar-minimal-wrapper");
        expanded.style_context().add_class("bar-expanded-wrapper");

        let (tx, _) = tokio::sync::broadcast::channel::<(i32, i32)>(16);
        let module = BarModule {
            tx,
            minimal,
            expanded,
            minimal_provider,
            expanded_provider,
            is_expanded: Rc::new(RefCell::new(false))
        };

        // hide expanded after we get it's allocation and set the style for it
        gtk4::glib::spawn_future_local({
            let expanded = module.expanded.clone();
            let expanded_provider = module.expanded_provider.clone();
            let mut rx = module.tx.subscribe();
            async move {
                if let Ok((width, height)) = rx.recv().await {
                    expanded_provider.load_from_data(&format!(
                        ".bar-expanded-wrapper {{
                            opacity: 0;
                            transform: scale({DOWNSCALE_FACTOR});
                            filter: blur({BLUR_FACTOR_PX}px);
                            margin: -{}px -{}px -{}px -{}px;
                        }}",
                        (height as f64)/2.0,
                        (width as f64)/2.0,
                        (height as f64)/2.0,
                        (width as f64)/2.0
                    ));
                    expanded.set_visible(false);
                }
            }
        });

        module.connect_expanded_map();

        module
    }

    pub fn is_expanded(&self) -> bool {
        *self.is_expanded.borrow()
    }

    pub fn toggle_expanded(&self) {
        let expanding = !self.is_expanded();
        *self.is_expanded.borrow_mut() = expanding;
        self.animate_fade_slide_down(expanding);
    }

    fn connect_expanded_map(&self) {
        self.expanded.connect_map({
            let expanded = self.expanded.clone();
            let tx = self.tx.clone();
            move |_| {
                gtk4::glib::spawn_future_local({
                    let expanded = expanded.clone();
                    let tx = tx.clone();
                    async move {
                        while expanded.allocated_width() == 0 || expanded.allocated_height() == 0 {
                            gtk4::glib::timeout_future(std::time::Duration::from_millis(1)).await;
                        }

                        let _ = tx.send((expanded.allocated_width(), expanded.allocated_height()));
                    }
                });
            }
        });
    }

    fn animate_fade_slide_down(&self, expanding: bool) {
        if expanding {
            self.expanded.set_visible(true);
        } else {
            self.minimal.set_visible(true);
        }

        let minimal_width = self.minimal.allocated_width() as f64;
        let expanded_width = self.expanded.allocated_width() as f64;
        let minimal_height = self.minimal.allocated_height() as f64;
        let expanded_height = self.expanded.allocated_height() as f64;
        
        if expanding {
            self.minimal_provider.load_from_data(&format!(
                ".bar-minimal-wrapper {{
                    opacity: 0;
                    transform: scale({DOWNSCALE_FACTOR});
                    filter: blur({BLUR_FACTOR_PX}px);
                    margin: -{}px -{}px -{}px -{}px;
                    transition-duration: {TRANSITION_DURATION}s;
                }}",
                minimal_height/2.0, minimal_width/2.0, minimal_height/2.0, minimal_width/2.0
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
    
        gtk4::glib::timeout_add_local_once(std::time::Duration::from_secs_f64(TRANSITION_DURATION), {
            let minimal = self.minimal.clone();
            let expanded = self.expanded.clone();
            let is_expanded = self.is_expanded.clone();
            move || {
                if expanding && is_expanded.borrow().to_owned() {
                    minimal.set_visible(false);
                } else if !is_expanded.borrow().to_owned() {
                    expanded.set_visible(false);
                }
            }
        });
    }
}
