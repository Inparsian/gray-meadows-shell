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

pub fn intercept_event(event: WpEvent) {
    match event {
        WpEvent::UpdateNode(id, property_name) => if let Some(node) = get_node(id) {
            let _ = NODES.get().map(|nodes| {
                if let Ok(mut nodes) = nodes.write()
                    && let Some(existing_node) = nodes.iter_mut().find(|n| n.id == node.id)
                {
                    match property_name.as_str() {
                        "description" => existing_node.description = ffi::node_get_description(id),
                        "icon" => existing_node.icon = ffi::node_get_icon(id),
                        "mute" => existing_node.mute = ffi::node_get_mute(id),
                        "name" => existing_node.name = ffi::node_get_name(id),
                        "path" => existing_node.path = ffi::node_get_path(id),
                        "serial" => existing_node.serial = ffi::node_get_serial(id),
                        "volume" => existing_node.volume = ffi::node_get_volume(id),
                        _ => {}
                    }
                }
            });
        },

        WpEvent::UpdateEndpoint(id, property_name) => if let Some(endpoint) = get_endpoint(id) {
            let _ = ENDPOINTS.get().map(|endpoints| if let Ok(mut endpoints) = endpoints.write() {
                let endpoint_node_id = endpoint.node.id;
                
                if let Some(endpoint_index) = endpoints.iter().position(|e| e.node.id == endpoint_node_id) {
                    match property_name.as_str() {
                        "is-default" => {
                            // Mutate all of the other endpoints to not be default.
                            for (i, e) in endpoints.iter_mut().enumerate() {
                                if i != endpoint_index {
                                    e.is_default = false;
                                }
                            }
                            
                            endpoints[endpoint_index].is_default = ffi::endpoint_get_is_default(id);
                        },
                        // Node is Endpoint's ancestor, so updates for the underlying node will be handled
                        // here as well.
                        "description" => endpoints[endpoint_index].node.description = ffi::node_get_description(id),
                        "icon" => endpoints[endpoint_index].node.icon = ffi::node_get_icon(id),
                        "mute" => endpoints[endpoint_index].node.mute = ffi::node_get_mute(id),
                        "name" => endpoints[endpoint_index].node.name = ffi::node_get_name(id),
                        "path" => endpoints[endpoint_index].node.path = ffi::node_get_path(id),
                        "serial" => endpoints[endpoint_index].node.serial = ffi::node_get_serial(id),
                        "volume" => endpoints[endpoint_index].node.volume = ffi::node_get_volume(id),
                        _ => {}
                    }
                }
            });
        },

        WpEvent::CreateStream(node) => {
            let _ = NODES.get().map(|nodes| {
                if let Ok(mut nodes) = nodes.write() {
                    nodes.push(node.clone());
                }
            });
        },

        WpEvent::RemoveStream(node) => {
            let _ = NODES.get().map(|nodes| {
                if let Ok(mut nodes) = nodes.write() {
                    nodes.retain(|n| n.id != node.id);
                }
            });
        },

        WpEvent::CreateMicrophone(endpoint) => {
            let _ = ENDPOINTS.get().map(|endpoints| {
                if let Ok(mut endpoints) = endpoints.write() {
                    endpoints.push(endpoint.clone());
                }
            });
        },

        WpEvent::RemoveMicrophone(endpoint) => {
            let _ = ENDPOINTS.get().map(|endpoints| {
                if let Ok(mut endpoints) = endpoints.write() {
                    endpoints.retain(|e| e.node.id != endpoint.node.id);
                }
            });
        },

        WpEvent::CreateSpeaker(endpoint) => {
            let _ = ENDPOINTS.get().map(|endpoints| {
                if let Ok(mut endpoints) = endpoints.write() {
                    endpoints.push(endpoint.clone());
                }
            });
        },

        WpEvent::RemoveSpeaker(endpoint) => {
            let _ = ENDPOINTS.get().map(|endpoints| {
                if let Ok(mut endpoints) = endpoints.write() {
                    endpoints.retain(|e| e.node.id != endpoint.node.id);
                }
            });
        },

        _ => {}
    }
}

pub fn activate() {
    // These vecs will start out empty and be populated by AstalWp later.
    let _ = NODES.set(RwLock::new(Vec::new()));
    let _ = ENDPOINTS.set(RwLock::new(Vec::new()));

    // Run the WirePlumber main loop in a separate thread.
    std::thread::spawn(ffi::init);
}