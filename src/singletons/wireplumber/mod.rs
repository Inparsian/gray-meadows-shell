use std::sync::Mutex;
use once_cell::sync::{OnceCell, Lazy};
use tokio::sync::broadcast;

use crate::ffi::astalwp::{ffi, CHANNEL, WpEvent};

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

static SENDER: Lazy<broadcast::Sender<WpEvent>> = Lazy::new(|| broadcast::channel(1).0);
static NODES: OnceCell<Mutex<Vec<ffi::Node>>> = once_cell::sync::OnceCell::new();
static ENDPOINTS: OnceCell<Mutex<Vec<ffi::Endpoint>>> = once_cell::sync::OnceCell::new();

pub fn subscribe() -> tokio::sync::broadcast::Receiver<WpEvent> {
    SENDER.subscribe()
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
                            let mut nodes = nodes.lock().unwrap();

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
                        });
                    }
                    
                    if POSSIBLE_NODE_PROPERTIES.contains(&property_name.as_str()) {
                        let _ = SENDER.send(WpEvent::UpdateNode(id, property_name));
                    }
                },

                WpEvent::UpdateEndpoint(id, property_name) => {
                    if let Some(endpoint) = get_endpoint(id) {
                        let _ = ENDPOINTS.get().map(|endpoints| {
                            if let Ok(mut endpoints) = endpoints.try_lock() {
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
                            let _ = SENDER.send(WpEvent::UpdateEndpoint(id, property_name));
                        }
                    }
                },

                WpEvent::UpdateDefaultMicrophone(id) => {
                    let _ = SENDER.send(WpEvent::UpdateDefaultMicrophone(id));
                },

                WpEvent::UpdateDefaultSpeaker(id) => {
                    let _ = SENDER.send(WpEvent::UpdateDefaultSpeaker(id));
                },

                WpEvent::CreateStream(node) => {
                    let node_cloned = node.clone();
                    let _ = NODES.get().map(|nodes| {
                        nodes.lock().unwrap().push(node);
                    });

                    let _ = SENDER.send(WpEvent::CreateStream(node_cloned));
                },

                WpEvent::RemoveStream(node) => {
                    let _ = NODES.get().map(|nodes| {
                        nodes.lock().unwrap().retain(|n| n.id != node.id);
                    });

                    let _ = SENDER.send(WpEvent::RemoveStream(node));
                },

                WpEvent::CreateMicrophone(endpoint) => {
                    let endpoint_cloned = endpoint.clone();
                    let _ = ENDPOINTS.get().map(|endpoints| {
                        endpoints.lock().unwrap().push(endpoint);
                    });

                    let _ = SENDER.send(WpEvent::CreateMicrophone(endpoint_cloned));
                },

                WpEvent::RemoveMicrophone(endpoint) => {
                    let _ = ENDPOINTS.get().map(|endpoints| {
                        endpoints.lock().unwrap().retain(|e| e.node.id != endpoint.node.id);
                    });

                    let _ = SENDER.send(WpEvent::RemoveMicrophone(endpoint));
                },

                WpEvent::CreateSpeaker(endpoint) => {
                    let endpoint_cloned = endpoint.clone();
                    let _ = ENDPOINTS.get().map(|endpoints| {
                        endpoints.lock().unwrap().push(endpoint);
                    });

                    let _ = SENDER.send(WpEvent::CreateSpeaker(endpoint_cloned));
                },

                WpEvent::RemoveSpeaker(endpoint) => {
                    let _ = ENDPOINTS.get().map(|endpoints| {
                        endpoints.lock().unwrap().retain(|e| e.node.id != endpoint.node.id);
                    });

                    let _ = SENDER.send(WpEvent::RemoveSpeaker(endpoint));
                }
            }
        }
    });

    // Run the WirePlumber main loop in a separate thread.
    std::thread::spawn(ffi::init);
}