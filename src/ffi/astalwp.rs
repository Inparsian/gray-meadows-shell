use std::sync::LazyLock;

use crate::utils::broadcast::BroadcastChannel;
use crate::singletons::wireplumber::intercept_event;

pub static CHANNEL: LazyLock<BroadcastChannel<WpEvent>> = LazyLock::new(|| BroadcastChannel::new(10));

#[derive(Debug, Clone)]
pub enum WpEvent {
    UpdateNode(i32, String),
    UpdateEndpoint(i32, String),
    UpdateDefaultMicrophone(i32),
    UpdateDefaultSpeaker(i32),
    CreateStream(ffi::Node),
    RemoveStream(ffi::Node),
    CreateRecorder(ffi::Node),
    RemoveRecorder(ffi::Node),
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
        Unknown,
    }

    #[derive(Debug, Clone)]
    struct Endpoint {
        type_: EndpointType,
        is_default: bool,
        node: Node,
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
        volume: f32,
    }

    extern "Rust" {
        pub fn receive_update_node(id: i32, property_name: String);
        pub fn receive_update_microphone(id: i32, property_name: String);
        pub fn receive_update_speaker(id: i32, property_name: String);
        pub fn receive_create_stream(node: Node);
        pub fn receive_remove_stream(node: Node);
        pub fn receive_create_recorder(node: Node);
        pub fn receive_remove_recorder(node: Node);
        pub fn receive_create_microphone(endpoint: Endpoint);
        pub fn receive_remove_microphone(endpoint: Endpoint);
        pub fn receive_create_speaker(endpoint: Endpoint);
        pub fn receive_remove_speaker(endpoint: Endpoint);
    }

    unsafe extern "C++" {
        include!("main.h");

        pub fn init();

        pub fn node_get_description(id: i32) -> String;
        pub fn node_get_icon(id: i32) -> String;
        #[allow(dead_code)] pub fn node_get_id(id: i32) -> i32;
        pub fn node_get_mute(id: i32) -> bool;
        pub fn node_get_name(id: i32) -> String;
        pub fn node_get_path(id: i32) -> String;
        pub fn node_get_serial(id: i32) -> i32;
        pub fn node_get_volume(id: i32) -> f32;
        pub fn node_set_mute(id: i32, mute: bool);
        pub fn node_set_volume(id: i32, volume: f32);

        pub fn endpoint_get_is_default(id: i32) -> bool;
        #[allow(dead_code)] pub fn endpoint_set_is_default(id: i32, is_default: bool);
    }
}

fn broadcast(event: WpEvent) {
    intercept_event(event.clone());
    CHANNEL.spawn_send(event);
}

fn receive_update_node(id: i32, property_name: String) {
    broadcast(WpEvent::UpdateNode(id, property_name));
}

fn receive_update_microphone(id: i32, property_name: String) {
    if property_name == "is-default" && ffi::endpoint_get_is_default(id) {
        broadcast(WpEvent::UpdateDefaultMicrophone(id));
    }
    
    broadcast(WpEvent::UpdateEndpoint(id, property_name));
}

fn receive_update_speaker(id: i32, property_name: String) {
    if property_name == "is-default" && ffi::endpoint_get_is_default(id) {
        broadcast(WpEvent::UpdateDefaultSpeaker(id));
    }

    broadcast(WpEvent::UpdateEndpoint(id, property_name));
}

fn receive_create_stream(node: ffi::Node) {
    broadcast(WpEvent::CreateStream(node));
}

fn receive_remove_stream(node: ffi::Node) {
    broadcast(WpEvent::RemoveStream(node));
}

fn receive_create_recorder(node: ffi::Node) {
    broadcast(WpEvent::CreateRecorder(node));
}

fn receive_remove_recorder(node: ffi::Node) {
    broadcast(WpEvent::RemoveRecorder(node));
}

fn receive_create_microphone(endpoint: ffi::Endpoint) {
    broadcast(WpEvent::CreateMicrophone(endpoint));
}

fn receive_remove_microphone(endpoint: ffi::Endpoint) {
    broadcast(WpEvent::RemoveMicrophone(endpoint));
}

fn receive_create_speaker(endpoint: ffi::Endpoint) {
    broadcast(WpEvent::CreateSpeaker(endpoint));
}

fn receive_remove_speaker(endpoint: ffi::Endpoint) {
    broadcast(WpEvent::RemoveSpeaker(endpoint));
}