#![allow(dead_code)]
use crate::ipc::{listen_for_messages, listen_for_messages_local};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right
}

pub enum MouseEvent {
    Press(MouseButton),
    Release(MouseButton),
}

pub fn listen_for_mouse_events<F>(callback: F)
where
    F: Fn(MouseEvent) + Send + 'static,
{
    listen_for_messages(move |raw| {
        match raw.as_str() {
            "mouse_left_press" => callback(MouseEvent::Press(MouseButton::Left)),
            "mouse_left_release" => callback(MouseEvent::Release(MouseButton::Left)),
            "mouse_right_press" => callback(MouseEvent::Press(MouseButton::Right)),
            "mouse_right_release" => callback(MouseEvent::Release(MouseButton::Right)),
            _ => {},
        }
    });
}

pub fn listen_for_mouse_events_local<F>(callback: F)
where
    F: Fn(MouseEvent) + 'static,
{
    listen_for_messages_local(move |raw| {
        match raw.as_str() {
            "mouse_left_press" => callback(MouseEvent::Press(MouseButton::Left)),
            "mouse_left_release" => callback(MouseEvent::Release(MouseButton::Left)),
            "mouse_right_press" => callback(MouseEvent::Press(MouseButton::Right)),
            "mouse_right_release" => callback(MouseEvent::Release(MouseButton::Right)),
            _ => {},
        }
    });
}