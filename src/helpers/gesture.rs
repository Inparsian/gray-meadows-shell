use gdk4::Key;
use gtk4::{prelude::GestureSingleExt, EventControllerScrollFlags};

pub fn on_key_press<F>(on_press: F) -> gtk4::EventControllerKey
where
    F: Fn(Key, u32) + 'static,
{
    let controller = gtk4::EventControllerKey::new();

    controller.connect_key_pressed(move |_, keyval, keycode, _| {
        on_press(keyval, keycode);
        gtk4::glib::Propagation::Proceed
    });

    controller
}

pub fn on_vertical_scroll<F>(on_scroll: F) -> gtk4::EventControllerScroll
where
    F: Fn(f64) + 'static,
{
    let controller = gtk4::EventControllerScroll::new(EventControllerScrollFlags::VERTICAL);

    controller.connect_scroll(move |_, _, dy| {
        on_scroll(dy);

        gtk4::glib::Propagation::Stop
    });

    controller
}

pub fn on_motion<F>(on_motion: F) -> gtk4::EventControllerMotion
where
    F: Fn(f64, f64) + 'static,
{
    let controller = gtk4::EventControllerMotion::new();

    controller.connect_motion(move |_, x, y| {
        on_motion(x, y);
    });

    controller
}

fn on_button_down<F>(button: u32, on_click: F) -> gtk4::GestureClick
where
    F: Fn(i32, f64, f64) + 'static,
{
    let controller = gtk4::GestureClick::new();
    controller.set_button(button);
    controller.connect_pressed(move |_, n_press, x, y| {
        on_click(n_press, x, y);
    });

    controller
}

fn on_button_up<F>(button: u32, on_click: F) -> gtk4::GestureClick
where
    F: Fn(i32, f64, f64) + 'static,
{
    let controller = gtk4::GestureClick::new();
    controller.set_button(button);
    controller.connect_released(move |_, n_press, x, y| {
        on_click(n_press, x, y);
    });

    controller
}

pub fn on_primary_down<F>(on_down: F) -> gtk4::GestureClick
where
    F: Fn(i32, f64, f64) + 'static,
{
    on_button_down(gdk4::ffi::GDK_BUTTON_PRIMARY.try_into().unwrap(), on_down)
}

pub fn on_primary_up<F>(on_up: F) -> gtk4::GestureClick
where
    F: Fn(i32, f64, f64) + 'static,
{
    on_button_up(gdk4::ffi::GDK_BUTTON_PRIMARY.try_into().unwrap(), on_up)
}

pub fn on_secondary_up<F>(on_up: F) -> gtk4::GestureClick
where
    F: Fn(i32, f64, f64) + 'static,
{
    on_button_up(gdk4::ffi::GDK_BUTTON_SECONDARY.try_into().unwrap(), on_up)
}

pub fn on_middle_up<F>(on_up: F) -> gtk4::GestureClick
where
    F: Fn(i32, f64, f64) + 'static,
{
    on_button_up(gdk4::ffi::GDK_BUTTON_MIDDLE.try_into().unwrap(), on_up)
}