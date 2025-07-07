use once_cell::sync::Lazy;
use tokio::sync::broadcast;

pub static CHANNEL: Lazy<broadcast::Sender<WpEvent>> = Lazy::new(|| {
    let (sender, _) = broadcast::channel(100);
    sender
});

#[derive(Clone)]
pub enum WpEvent {
    UpdateNode(ffi::Node, String),
    UpdateEndpoint(ffi::Endpoint, String),
    CreateStream(ffi::Node),
    RemoveStream(ffi::Node),
    CreateMicrophone(ffi::Endpoint),
    RemoveMicrophone(ffi::Endpoint),
    CreateSpeaker(ffi::Endpoint),
    RemoveSpeaker(ffi::Endpoint),
}

#[allow(clippy::module_inception)]
#[cxx::bridge]
pub mod ffi {
    #[derive(Clone)]
    struct Endpoint {
        is_default: bool,
        node: Node
    }

    #[derive(Clone)]
    struct Node {
        description: String,
        icon: String,
        id: i32,
        mute: bool,
        name: String,
        path: String,
        serial: i32,
        volume: f32
    }

    extern "Rust" {
        pub fn receive_update_node(node: Node, property_name: String);
        pub fn receive_update_endpoint(endpoint: Endpoint, property_name: String);
        pub fn receive_create_stream(node: Node);
        pub fn receive_remove_stream(node: Node);
        pub fn receive_create_microphone(endpoint: Endpoint);
        pub fn receive_remove_microphone(endpoint: Endpoint);
        pub fn receive_create_speaker(endpoint: Endpoint);
        pub fn receive_remove_speaker(endpoint: Endpoint);
    }

    unsafe extern "C++" {
        include!("main.h");

        pub fn init();

        #[allow(dead_code)] pub fn node_get_description(id: i32) -> String;
        #[allow(dead_code)] pub fn node_get_icon(id: i32) -> String;
        #[allow(dead_code)] pub fn node_get_id(id: i32) -> i32;
        #[allow(dead_code)] pub fn node_get_mute(id: i32) -> bool;
        #[allow(dead_code)] pub fn node_get_name(id: i32) -> String;
        #[allow(dead_code)] pub fn node_get_path(id: i32) -> String;
        #[allow(dead_code)] pub fn node_get_serial(id: i32) -> i32;
        #[allow(dead_code)] pub fn node_get_volume(id: i32) -> f32;
        #[allow(dead_code)] pub fn node_set_mute(id: i32, mute: bool);
        #[allow(dead_code)] pub fn node_set_volume(id: i32, volume: f32);

        #[allow(dead_code)] pub fn endpoint_get_is_default(id: i32) -> bool;
        #[allow(dead_code)] pub fn endpoint_set_is_default(id: i32, is_default: bool);
    }
}

pub fn receive_update_node(node: ffi::Node, property_name: String) {
    let _ = CHANNEL.send(WpEvent::UpdateNode(
        node,
        property_name,
    ));
}

pub fn receive_update_endpoint(endpoint: ffi::Endpoint, property_name: String) {
    let _ = CHANNEL.send(WpEvent::UpdateEndpoint(
        endpoint,
        property_name,
    ));
}

pub fn receive_create_stream(node: ffi::Node) {
    let _ = CHANNEL.send(WpEvent::CreateStream(
        node,
    ));
}

pub fn receive_remove_stream(node: ffi::Node) {
    let _ = CHANNEL.send(WpEvent::RemoveStream(
        node,
    ));
}

pub fn receive_create_microphone(endpoint: ffi::Endpoint) {
    let _ = CHANNEL.send(WpEvent::CreateMicrophone(
        endpoint,
    ));
}

pub fn receive_remove_microphone(endpoint: ffi::Endpoint) {
    let _ = CHANNEL.send(WpEvent::RemoveMicrophone(
        endpoint,
    ));
}

pub fn receive_create_speaker(endpoint: ffi::Endpoint) {
    let _ = CHANNEL.send(WpEvent::CreateSpeaker(
        endpoint,
    ));
}

pub fn receive_remove_speaker(endpoint: ffi::Endpoint) {
    let _ = CHANNEL.send(WpEvent::RemoveSpeaker(
        endpoint,
    ));
}