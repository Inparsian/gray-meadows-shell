use std::sync::{Mutex, OnceLock, LazyLock};
use async_broadcast::Receiver;

use crate::{broadcast::BroadcastChannel, ffi::astalwp::{CHANNEL, WpEvent, ffi}};

const POSSIBLE_NODE_PROPERTIES: [&str; 7] = [
    "description",
    "icon",
    "mute",
    "name",
    "path",
    "serial",
    "volume",
];

const POSSIBLE_ENDPOINT_PROPERTIES: [&str; 1] = [
    "is-default"
];

static EVENT_CHANNEL: LazyLock<BroadcastChannel<WpEvent>> = LazyLock::new(|| BroadcastChannel::new(10));
static NODES: OnceLock<Mutex<Vec<ffi::Node>>> = OnceLock::new();
static ENDPOINTS: OnceLock<Mutex<Vec<ffi::Endpoint>>> = OnceLock::new();

pub fn subscribe() -> Receiver<WpEvent> {
    EVENT_CHANNEL.subscribe()
}

pub fn subscribe_default_speaker_volume<F>(callback: F)
where 
    F: Fn(f32) + 'static
{
    gtk4::glib::spawn_future_local(async move {
        while let Ok(event) = subscribe().recv().await {
            match event {
                WpEvent::CreateSpeaker(endpoint) => {
                    if endpoint.is_default {
                        callback(endpoint.node.volume);
                    }
                },
            
                WpEvent::RemoveSpeaker(endpoint) => {
                    if let Some(default_speaker) = get_default_speaker() {
                        if default_speaker.node.id == endpoint.node.id {
                            callback(default_speaker.node.volume);
                        }
                    }
                },
            
                WpEvent::UpdateDefaultSpeaker(id) => {
                    if let Some(speaker) = get_endpoint(id) {
                        callback(speaker.node.volume);
                    }
                },
            
                WpEvent::UpdateEndpoint(id, property_name) => if property_name == "volume" {
                    if let Some(default_speaker) = get_default_speaker() {
                        if default_speaker.node.id == id {
                            callback(default_speaker.node.volume);
                        }
                    }
                },
            
                _ => {}
            }
        }
    });
}

pub fn get_node(id: i32) -> Option<ffi::Node> {
    NODES.get()?.try_lock().ok()?.iter().find(|&n| n.id == id).cloned()
}

pub fn get_endpoint(id: i32) -> Option<ffi::Endpoint> {
    ENDPOINTS.get()?.try_lock().ok()?.iter().find(|&e| e.node.id == id).cloned()
}

pub fn get_default_speaker() -> Option<ffi::Endpoint> {
    ENDPOINTS.get()?.try_lock().ok()?.iter().find(|&e| e.is_default && e.type_ == ffi::EndpointType::Speaker).cloned()
}

pub fn activate() {
    // These vecs will start out empty and be populated by AstalWp later.
    let _ = NODES.set(Mutex::new(Vec::new()));
    let _ = ENDPOINTS.set(Mutex::new(Vec::new()));

    let mut receiver = CHANNEL.subscribe();
    tokio::spawn(async move {
        while let Ok(event) = receiver.recv().await {
            match event {
                WpEvent::UpdateNode(id, property_name) => {
                    if let Some(node) = get_node(id) {
                        let _ = NODES.get().map(|nodes| {
                            if let Ok(mut nodes) = nodes.lock() {
                                if let Some(existing_node) = nodes.iter_mut().find(|n| n.id == node.id) {
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
                            }
                        });
                    }
                    
                    if POSSIBLE_NODE_PROPERTIES.contains(&property_name.as_str()) {
                        EVENT_CHANNEL.send(WpEvent::UpdateNode(id, property_name)).await;
                    }
                },

                WpEvent::UpdateEndpoint(id, property_name) => {
                    if let Some(endpoint) = get_endpoint(id) {
                        let _ = ENDPOINTS.get().map(|endpoints| {
                            if let Ok(mut endpoints) = endpoints.lock() {
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
                            }
                        });

                        if POSSIBLE_ENDPOINT_PROPERTIES.contains(&property_name.as_str()) || POSSIBLE_NODE_PROPERTIES.contains(&property_name.as_str()) {
                            EVENT_CHANNEL.send(WpEvent::UpdateEndpoint(id, property_name)).await;
                        }
                    }
                },

                WpEvent::UpdateDefaultMicrophone(id) => {
                    EVENT_CHANNEL.send(WpEvent::UpdateDefaultMicrophone(id)).await;
                },

                WpEvent::UpdateDefaultSpeaker(id) => {
                    EVENT_CHANNEL.send(WpEvent::UpdateDefaultSpeaker(id)).await;
                },

                WpEvent::CreateStream(node) => {
                    let _ = NODES.get().map(|nodes| {
                        if let Ok(mut nodes) = nodes.lock() {
                            nodes.push(node.clone());
                        }
                    });

                    EVENT_CHANNEL.send(WpEvent::CreateStream(node)).await;
                },

                WpEvent::RemoveStream(node) => {
                    let _ = NODES.get().map(|nodes| {
                        if let Ok(mut nodes) = nodes.lock() {
                            nodes.retain(|n| n.id != node.id);
                        }
                    });

                    EVENT_CHANNEL.send(WpEvent::RemoveStream(node)).await;
                },

                WpEvent::CreateMicrophone(endpoint) => {
                    let _ = ENDPOINTS.get().map(|endpoints| {
                        if let Ok(mut endpoints) = endpoints.lock() {
                            endpoints.push(endpoint.clone());
                        }
                    });

                    EVENT_CHANNEL.send(WpEvent::CreateMicrophone(endpoint)).await;
                },

                WpEvent::RemoveMicrophone(endpoint) => {
                    let _ = ENDPOINTS.get().map(|endpoints| {
                        if let Ok(mut endpoints) = endpoints.lock() {
                            endpoints.retain(|e| e.node.id != endpoint.node.id);
                        }
                    });

                    EVENT_CHANNEL.send(WpEvent::RemoveMicrophone(endpoint)).await;
                },

                WpEvent::CreateSpeaker(endpoint) => {
                    let _ = ENDPOINTS.get().map(|endpoints| {
                        if let Ok(mut endpoints) = endpoints.lock() {
                            endpoints.push(endpoint.clone());
                        }
                    });

                    EVENT_CHANNEL.send(WpEvent::CreateSpeaker(endpoint)).await;
                },

                WpEvent::RemoveSpeaker(endpoint) => {
                    let _ = ENDPOINTS.get().map(|endpoints| {
                        if let Ok(mut endpoints) = endpoints.lock() {
                            endpoints.retain(|e| e.node.id != endpoint.node.id);
                        }
                    });

                    EVENT_CHANNEL.send(WpEvent::RemoveSpeaker(endpoint)).await;
                }
            }
        }
    });

    // Run the WirePlumber main loop in a separate thread.
    std::thread::spawn(ffi::init);
}