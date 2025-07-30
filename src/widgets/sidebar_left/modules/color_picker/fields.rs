use std::{cell::RefCell, rc::Rc, time::Duration};
use gtk4::prelude::*;

static LOCK_HOLD_DURATION: Duration = Duration::from_millis(2);

#[derive(Debug, Clone)]
pub enum FieldType {
    Entry,
    SpinButton(gtk4::Adjustment)
}

#[derive(Debug, Clone)]
pub enum FieldUpdate {
    Text(String),
    Float(f64)
}

#[derive(Debug, Clone)]
pub struct Field {
    pub widget: gtk4::Widget,
    pub lock: Rc<RefCell<bool>>
}

impl Field {
    pub fn new<F>(field_type: FieldType, update_callback: F) -> Self
    where
        F: Fn(FieldUpdate) + 'static + Send + Sync,
    {
        let lock = Rc::new(RefCell::new(false));

        let widget: gtk4::Widget = match field_type {
            FieldType::Entry => {
                let entry = gtk4::Entry::new();
                entry.set_css_classes(&["color-picker-entry"]);

                entry.connect_changed({
                    let lock = lock.clone();
                    move |entry| if !lock.try_borrow().as_deref().unwrap_or(&true) {
                        activate_lock(&lock);
                        update_callback(FieldUpdate::Text(entry.text().to_string()));
                    }
                });

                entry.upcast()
            },

            FieldType::SpinButton(adjustment) => {
                let spin_button = gtk4::SpinButton::new(Some(&adjustment), 1.0, 0);
                spin_button.set_css_classes(&["color-picker-spinbutton"]);

                spin_button.connect_value_changed({
                    let lock = lock.clone();
                    move |spin_button| if !lock.try_borrow().as_deref().unwrap_or(&true) {
                        activate_lock(&lock);
                        update_callback(FieldUpdate::Float(spin_button.value()));
                    }
                });

                spin_button.upcast()
            }
        };

        Self {
            widget,
            lock
        }
    }

    pub fn is_locked(&self) -> bool {
        *self.lock.try_borrow().as_deref().unwrap_or(&true)
    }

    pub fn lock(&self) {
        if self.is_locked() {
            return;
        }

        activate_lock(&self.lock);
    }
}

#[derive(Debug, Clone)]
pub struct Fields {
    pub widget: gtk4::Box,
    pub fields: Vec<Field>
}

impl Fields {
    pub fn new() -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        widget.set_css_classes(&["color-picker-fields-box"]);
        widget.set_hexpand(true);
        widget.set_valign(gtk4::Align::Start);
        widget.set_homogeneous(true);
        widget.set_spacing(8);

        Self {
            widget,
            fields: Vec::new()
        }
    }

    pub fn add_field<F>(&mut self, field_type: FieldType, update_callback: F)
    where
        F: Fn(FieldUpdate) + 'static + Send + Sync,
    {
        let field = Field::new(field_type, update_callback);
        self.widget.append(&field.widget);
        self.fields.push(field);
    }

    pub fn update(&self, updates: Vec<FieldUpdate>) {
        // One update is for each field
        if updates.len() != self.fields.len() {
            return;
        }

        for (i, update) in updates.into_iter().enumerate() {
            if let Some(field) = self.fields.get(i) {
                if !field.is_locked() {
                    field.lock();
                    
                    match update {
                        FieldUpdate::Text(text) => {
                            if let Some(entry) = field.widget.downcast_ref::<gtk4::Entry>() {
                                entry.set_text(&text);
                            }
                        },

                        FieldUpdate::Float(value) => {
                            if let Some(spin_button) = field.widget.downcast_ref::<gtk4::SpinButton>() {
                                spin_button.set_value(value);
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn activate_lock(lock: &Rc<RefCell<bool>>) {
    *lock.borrow_mut() = true;

    // Unlock after LOCK_HOLD_DURATION
    gtk4::glib::timeout_add_local_once(LOCK_HOLD_DURATION, {
        let lock = lock.clone();
        move || *lock.borrow_mut() = false
    });
}