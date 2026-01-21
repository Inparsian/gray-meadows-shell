// For the widgets I am too lazy to subclass :P
use std::rc::Rc;
use std::cell::{Cell, RefCell};
use glib::object::{IsA, ObjectExt as _};
use glib::WeakRef;
use gtk4::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct AllocationWatcherOptions {
    pub timeout_millis: u64,
    pub max_allocation_width: Option<i32>,
    pub max_allocation_height: Option<i32>,
    pub min_allocation_width: i32,
    pub min_allocation_height: i32,
}

pub struct AllocationWatcher<W: IsA<gtk4::Widget>> {
    pub options: AllocationWatcherOptions,
    pub widget: WeakRef<W>,
    pub tick: Rc<RefCell<Option<gtk4::TickCallbackId>>>,
    pub last_received_allocation: Rc<Cell<Option<gtk4::gdk::Rectangle>>>,
}

impl<W: IsA<gtk4::Widget>> AllocationWatcher<W> {
    pub fn new(widget: &W, options: AllocationWatcherOptions) -> Self {
        let weak = widget.downgrade();

        AllocationWatcher {
            options,
            widget: weak,
            tick: Rc::new(RefCell::new(None)),
            last_received_allocation: Rc::new(Cell::new(None)),
        }
    }

    pub fn is_watching(&self) -> bool {
        self.tick.borrow().is_some()
    }

    pub fn one_shot_future<R: 'static, F: std::future::Future<Output = R> + 'static>(&self, f: F) {
        let Some(widget) = self.widget.upgrade() else {
            return;
        };

        if self.is_watching() {
            return;
        }

        let old_allocation = {
            let allocation = widget.allocation();
            
            Rc::new(Cell::new((
                allocation.width(),
                allocation.height(),
            )))
        };

        let future_cell = Rc::new(RefCell::new(Some(f)));
        self.tick.borrow_mut().replace(widget.add_tick_callback({
            let tick = self.tick.clone();
            let max_allocation_width = self.options.max_allocation_width;
            let max_allocation_height = self.options.max_allocation_height;
            let min_allocation_width = self.options.min_allocation_width;
            let min_allocation_height = self.options.min_allocation_height;
            let future_cell = future_cell.clone();
            let last_received_allocation = self.last_received_allocation.clone();

            move |widget, _| {
                let (new_width, new_height) = {
                    let allocation = widget.allocation();
                    (allocation.width(), allocation.height())
                };

                let (old_width, old_height) = old_allocation.get();

                if new_width != old_width || new_height != old_height {
                    last_received_allocation.set(Some(widget.allocation()));

                    if new_width >= min_allocation_width && new_height >= min_allocation_height
                        && max_allocation_width.is_none_or(|max_w| new_width <= max_w)
                        && max_allocation_height.is_none_or(|max_h| new_height <= max_h)
                    {
                        tick.borrow_mut().take();
                        if let Some(future) = future_cell.borrow_mut().take() {
                            glib::spawn_future_local(future);
                        }

                        return glib::ControlFlow::Break;
                    }
                    
                    old_allocation.set((new_width, new_height));
                }
                
                glib::ControlFlow::Continue
            }
        }));

        // It is possible that the allocated size might not change at all, so the supplied future
        // would never run. For such cases, we'll stop watching if we do not detect an allocation
        // change.
        glib::spawn_future_local({
            let timeout_millis = self.options.timeout_millis;
            let tick = self.tick.clone();
            async move {
                glib::timeout_future(std::time::Duration::from_millis(timeout_millis)).await;

                if let Some(tick) = tick.borrow_mut().take() {
                    if let Some(future) = future_cell.borrow_mut().take() {
                        glib::spawn_future_local(future);
                    }

                    tick.remove();
                }
            }
        });
    }
}