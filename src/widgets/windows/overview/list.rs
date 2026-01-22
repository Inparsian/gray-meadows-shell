use std::{cell::RefCell, rc::Rc, time::Duration};
use gtk4::prelude::*;

use super::item::{OverviewSearchItem, ITEM_ANIMATION_DURATION};

static LOCK_HOLD_DURATION: Duration = Duration::from_millis(1);

#[derive(Debug, Clone)]
pub struct OverviewSearchList {
    pub items: Vec<OverviewSearchItem>,
    widget: gtk4::ListBox,
    lock: Rc<RefCell<bool>> // To prevent race conditions when the user types too fast
}

pub fn get_button_from_row(row: &gtk4::ListBoxRow) -> Option<gtk4::Button> {
    let mut current_widget: Option<gtk4::Widget> = Some(row.child()?);
    while let Some(widget) = current_widget {
        if let Some(button) = widget.downcast_ref::<gtk4::Button>() {
            return Some(button.clone());
        }

        if let Some(container) = widget.downcast_ref::<gtk4::Revealer>() {
            current_widget = container.child();
        } else {
            break;
        }
    }

    None
}

impl OverviewSearchList {
    pub fn new() -> Self {
        let widget = gtk4::ListBox::new();
        widget.set_selection_mode(gtk4::SelectionMode::Single);
        widget.set_css_classes(&["overview-search-results"]);

        widget.connect_row_activated(|_, row| if let Some(button) = get_button_from_row(row) {
            button.activate();
        });

        widget.connect_row_selected(|_, row| if let Some(button) = row.and_then(get_button_from_row) {
            button.grab_focus();
        });

        Self {
            items: Vec::new(),
            widget,
            lock: Rc::new(RefCell::new(false))
        }
    }

    pub fn get_widget(&self) -> gtk4::ListBox {
        self.widget.clone()
    }

    pub fn lock(&self) {
        *self.lock.borrow_mut() = true;

        // Unlock after LOCK_HOLD_DURATION
        glib::timeout_add_local_once(LOCK_HOLD_DURATION, {
            let lock = self.lock.clone();
            move || *lock.borrow_mut() = false
        });
    }

    pub fn insert(&mut self, item: &OverviewSearchItem, position: usize) {
        if self.lock.try_borrow().map_or(true, |lock| *lock) || position > self.items.len() {
            return;
        }

        self.widget.insert(&item.get_row(), position as i32);

        // Reveal this item after 1ms
        glib::timeout_add_local_once(Duration::from_millis(1), clone!(
            #[strong] item,
            move || item.reveal()
        ));

        self.items.insert(position, item.clone());
    }

    pub fn remove(&mut self, position: usize) {
        if self.lock.try_borrow().map_or(true, |lock| *lock) || position >= self.items.len() {
            return;
        }

        if position < self.items.len() {
            let item = self.items.remove(position);
            item.hide();

            glib::timeout_add_local_once(Duration::from_millis(ITEM_ANIMATION_DURATION as u64), {
                let widget = self.widget.clone();
                move || widget.remove(&item.get_row())
            });
        }
    }

    pub fn move_item(&mut self, from: usize, to: usize) {
        if from < self.items.len() && to < self.items.len() && from != to {
            let item = self.items.remove(from);
            self.items.insert(to, item);

            // Update the ListBox
            let row = self.widget.row_at_index(from as i32).unwrap();
            self.widget.remove(&row);
            self.widget.insert(&row, to as i32);
        }
    }
}