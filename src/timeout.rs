use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug)]
pub struct Timeout {
    source: Rc<RefCell<Option<gtk4::glib::SourceId>>>,
}

impl Default for Timeout {
    fn default() -> Self {
        Timeout {
            source: Rc::new(RefCell::new(None))
        }
    }
}

impl Timeout {
    pub fn set<F>(&self, duration: std::time::Duration, callback: F)
    where
        F: FnOnce() + 'static,
    {
        if let Ok(mut source) = self.source.try_borrow_mut() {
            if let Some(existing_source) = (*source).take() {
                existing_source.remove();
            }

            *source = Some(gtk4::glib::timeout_add_local_once(duration, {
                let source = self.source.clone();
                move || {
                    callback();
                    *source.borrow_mut() = None;
                }
            }));
        }
    }
}