use std::sync::{RwLock, OnceLock};
use async_broadcast::Receiver;

use crate::ffi::astalwp::{CHANNEL, WpEvent, ffi};

static NODES: OnceLock<RwLock<Vec<ffi::Node>>> = OnceLock::new();
static ENDPOINTS: OnceLock<RwLock<Vec<ffi::Endpoint>>> = OnceLock::new();

pub fn subscribe() -> Receiver<WpEvent> {
    CHANNEL.subscribe()
}

pub fn subscribe_default_speaker_volume<F>(callback: F)
where 
    F: Fn(f32) + 'static
{
    gtk4::glib::spawn_future_local(async move {
        let mut receiver = subscribe();
        while let Ok(event) = receiver.recv().await {
            match event {
                WpEvent::CreateSpeaker(endpoint) => if endpoint.is_default {
                    callback(endpoint.node.volume);
                },
            
                WpEvent::RemoveSpeaker(endpoint) => if let Some(default_speaker) = get_default_speaker()
                    && default_speaker.node.id == endpoint.node.id
                {
                    callback(default_speaker.node.volume);
                },
            
                WpEvent::UpdateDefaultSpeaker(id) => if let Some(speaker) = get_endpoint(id) {
                    callback(speaker.node.volume);
                },
            
                WpEvent::UpdateEndpoint(id, property_name) => if property_name == "volume"
                    && let Some(default_speaker) = get_default_speaker()
                    && default_speaker.node.id == id
                {
                    callback(default_speaker.node.volume);
                },
            
                _ => {}
            }
        }
    });
}

pub fn get_node(id: i32) -> Option<ffi::Node> {
    NODES.get()?.read().ok()?.iter().find(|&n| n.id == id).cloned()
}

pub fn get_endpoint(id: i32) -> Option<ffi::Endpoint> {
    ENDPOINTS.get()?.read().ok()?.iter().find(|&e| e.node.id == id).cloned()
}

pub fn get_default_speaker() -> Option<ffi::Endpoint> {
    ENDPOINTS.get()?.read().ok()?.iter().find(|&e| e.is_default && e.type_ == ffi::EndpointType::Speaker).cloned()
}

pub fn get_default_microphone() -> Option<ffi::Endpoint> {
    ENDPOINTS.get()?.read().ok()?.iter().find(|&e| e.is_default && e.type_ == ffi::EndpointType::Microphone).cloned()
}

pub fn update_node_property(node: &mut ffi::Node, property_name: &str) {
    match property_name {
        "description" => node.description = ffi::node_get_description(node.id),
        "icon" => node.icon = ffi::node_get_icon(node.id),
        "mute" => node.mute = ffi::node_get_mute(node.id),
        "name" => node.name = ffi::node_get_name(node.id),
        "path" => node.path = ffi::node_get_path(node.id),
        "serial" => node.serial = ffi::node_get_serial(node.id),
        "volume" => node.volume = ffi::node_get_volume(node.id),
        _ => {}
    }
}

pub fn intercept_event(event: WpEvent) {
    match event {
        WpEvent::UpdateNode(id, property_name) => if let Some(node) = get_node(id)
            && let Some(nodes) = NODES.get()
            && let Ok(mut nodes) = nodes.write()
            && let Some(existing_node) = nodes.iter_mut().find(|n| n.id == node.id)
        {
            update_node_property(existing_node, &property_name);
        },

        WpEvent::UpdateEndpoint(id, property_name) => if let Some(endpoint) = get_endpoint(id)
            && let Some(endpoints) = ENDPOINTS.get()
            && let Ok(mut endpoints) = endpoints.write()
            && let Some(existing_endpoint) = endpoints.iter_mut().find(|e| e.node.id == endpoint.node.id)
        {
            // Endpoint is a Node superset
            update_node_property(&mut existing_endpoint.node, &property_name);
        },
        
        WpEvent::UpdateDefaultMicrophone(id) | WpEvent::UpdateDefaultSpeaker(id) => {
            if let Some(endpoints) = ENDPOINTS.get() && let Ok(mut endpoints) = endpoints.write() {
                for e in endpoints.iter_mut() {
                    e.is_default = e.node.id == id;
                }
            }
        },

        WpEvent::CreateStream(node) | WpEvent::CreateRecorder(node) => {
            if let Some(nodes) = NODES.get() && let Ok(mut nodes) = nodes.write() {
                nodes.push(node);
            }
        },

        WpEvent::RemoveStream(node) | WpEvent::RemoveRecorder(node) => {
            if let Some(nodes) = NODES.get() && let Ok(mut nodes) = nodes.write() {
                nodes.retain(|n| n.id != node.id);
            }
        },

        WpEvent::CreateMicrophone(endpoint) | WpEvent::CreateSpeaker(endpoint) => {
            if let Some(endpoints) = ENDPOINTS.get() && let Ok(mut endpoints) = endpoints.write() {
                endpoints.push(endpoint);
            }
        },

        WpEvent::RemoveMicrophone(endpoint) | WpEvent::RemoveSpeaker(endpoint) => {
            if let Some(endpoints) = ENDPOINTS.get() && let Ok(mut endpoints) = endpoints.write() {
                endpoints.retain(|e| e.node.id != endpoint.node.id);
            }
        },
    }
}

pub fn activate() {
    // These vecs will start out empty and be populated by AstalWp later.
    let _ = NODES.set(RwLock::new(Vec::new()));
    let _ = ENDPOINTS.set(RwLock::new(Vec::new()));

    // Run the WirePlumber main loop in a separate thread.
    std::thread::spawn(ffi::init);
}