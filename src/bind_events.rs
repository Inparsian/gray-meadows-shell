#![allow(dead_code)]
use crate::ipc::listen_for_messages_local;

pub enum MouseEvent {
    LeftPress,
    LeftRelease,
    RightPress,
    RightRelease
}

pub fn listen_for_mouse_events_local<F>(callback: F)
where
    F: Fn(MouseEvent) + 'static,
{
    listen_for_messages_local(move |raw| {
        match raw.as_str() {
            "mouse_left_press" => callback(MouseEvent::LeftPress),
            "mouse_left_release" => callback(MouseEvent::LeftRelease),
            "mouse_right_press" => callback(MouseEvent::RightPress),
            "mouse_right_release" => callback(MouseEvent::RightRelease),
            _ => eprintln!("Unknown mouse event: {}", raw),
        }
    });
}