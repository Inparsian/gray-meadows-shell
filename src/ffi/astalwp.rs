use std::sync::LazyLock;

use crate::broadcast::BroadcastChannel;

pub static CHANNEL: LazyLock<BroadcastChannel<WpEvent>> = LazyLock::new(|| BroadcastChannel::new(10));

#[derive(Debug, Clone)]
pub enum WpEvent {
    UpdateNode(i32, String),
    UpdateEndpoint(i32, String),
    UpdateDefaultMicrophone(i32),
    UpdateDefaultSpeaker(i32),
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
    #[derive(Debug, Clone)]
    pub enum EndpointType {
        Microphone,
        Speaker,
        Unknown
    }

    #[derive(Debug, Clone)]
    struct Endpoint {
        type_: EndpointType,
        is_default: bool,
        node: Node
    }

    #[derive(Debug, Clone)]
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
        pub fn receive_update_node(id: i32, property_name: String);
        pub fn receive_update_microphone(id: i32, property_name: String);
        pub fn receive_update_speaker(id: i32, property_name: String);
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

pub fn receive_update_node(id: i32, property_name: String) {
    CHANNEL.spawn_send(WpEvent::UpdateNode(
        id,
        property_name,
    ));
}

pub fn receive_update_microphone(id: i32, property_name: String) {
    if property_name == "is-default" && ffi::endpoint_get_is_default(id) {
        CHANNEL.spawn_send(WpEvent::UpdateDefaultMicrophone(id));
    }
    
    CHANNEL.spawn_send(WpEvent::UpdateEndpoint(
        id,
        property_name,
    ));
}

pub fn receive_update_speaker(id: i32, property_name: String) {
    if property_name == "is-default" && ffi::endpoint_get_is_default(id) {
        CHANNEL.spawn_send(WpEvent::UpdateDefaultSpeaker(id));
    }

    CHANNEL.spawn_send(WpEvent::UpdateEndpoint(
        id,
        property_name,
    ));
}

pub fn receive_create_stream(node: ffi::Node) {
    CHANNEL.spawn_send(WpEvent::CreateStream(
        node,
    ));
}

pub fn receive_remove_stream(node: ffi::Node) {
    CHANNEL.spawn_send(WpEvent::RemoveStream(
        node,
    ));
}

pub fn receive_create_microphone(endpoint: ffi::Endpoint) {
    CHANNEL.spawn_send(WpEvent::CreateMicrophone(
        endpoint,
    ));
}

pub fn receive_remove_microphone(endpoint: ffi::Endpoint) {
    CHANNEL.spawn_send(WpEvent::RemoveMicrophone(
        endpoint,
    ));
}

pub fn receive_create_speaker(endpoint: ffi::Endpoint) {
    CHANNEL.spawn_send(WpEvent::CreateSpeaker(
        endpoint,
    ));
}

pub fn receive_remove_speaker(endpoint: ffi::Endpoint) {
    CHANNEL.spawn_send(WpEvent::RemoveSpeaker(
        endpoint,
    ));
}