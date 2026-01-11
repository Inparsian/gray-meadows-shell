// For the widgets I am too lazy to subclass :P
use std::rc::Rc;
use std::cell::{Cell, RefCell};
use gtk4::glib::object::{IsA, ObjectExt as _};
use gtk4::glib::WeakRef;
use gtk4::prelude::*;

pub struct AllocationWatcher<W: IsA<gtk4::Widget>> {
    pub widget: WeakRef<W>,
    pub watching: Rc<Cell<bool>>,
    pub last_received_allocation: Rc<Cell<Option<gtk4::gdk::Rectangle>>>,
}

impl<W: IsA<gtk4::Widget>> AllocationWatcher<W> {
    pub fn new(widget: &W) -> Self {
        let weak = widget.downgrade();

        AllocationWatcher {
            widget: weak,
            watching: Rc::new(Cell::new(false)),
            last_received_allocation: Rc::new(Cell::new(None)),
        }
    }

    pub fn is_watching(&self) -> bool {
        self.watching.get()
    }

    pub fn next_allocation_future<R: 'static, F: std::future::Future<Output = R> + 'static>(
        &self,
        timeout_millis: u64,
        invoke_after_change_amount: usize,
        f: F,
    ) {
        let Some(widget) = self.widget.upgrade() else {
            return;
        };

        if self.is_watching() {
            return;
        }
        
        self.watching.set(true);

        let old_allocation = {
            let allocation = widget.allocation();
            
            Rc::new(Cell::new((
                allocation.width(),
                allocation.height(),
            )))
        };

        let changes = Rc::new(Cell::new(0));
        let future_cell = Rc::new(RefCell::new(Some(f)));
        let tick = widget.add_tick_callback({
            let watching = self.watching.clone();
            let future_cell = future_cell.clone();
            let last_received_allocation = self.last_received_allocation.clone();

            move |widget, _| {
                let (new_width, new_height) = {
                    let allocation = widget.allocation();
                    (allocation.width(), allocation.height())
                };

                let (old_width, old_height) = old_allocation.get();

                if new_width != old_width || new_height != old_height {
                    changes.set(changes.get() + 1);

                    if changes.get() >= invoke_after_change_amount {
                        watching.set(false);
                        last_received_allocation.set(Some(widget.allocation()));

                        if let Some(future) = future_cell.borrow_mut().take() {
                            gtk4::glib::spawn_future_local(future);
                        }

                        return gtk4::glib::ControlFlow::Break;
                    }
                    
                    old_allocation.set((new_width, new_height));
                }
                
                gtk4::glib::ControlFlow::Continue
            }
        });

        // It is possible that the allocated size might not change at all, so the supplied future
        // would never run. For such cases, we'll stop watching if we do not detect an allocation
        // change.
        gtk4::glib::spawn_future_local({
            let watching = self.watching.clone();
            async move {
                gtk4::glib::timeout_future(std::time::Duration::from_millis(timeout_millis)).await;

                if watching.get() {
                    watching.set(false);

                    if let Some(future) = future_cell.borrow_mut().take() {
                        gtk4::glib::spawn_future_local(future);
                    }

                    tick.remove();
                }
            }
        });
    }
}