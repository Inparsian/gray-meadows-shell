use std::sync::Mutex;

use crate::ffi::astalwp::{ffi, CHANNEL, WpEvent};

const _POSSIBLE_NODE_PROPERTIES: [&str; 7] = [
    "description",
    "icon",
    "mute",
    "name",
    "path",
    "serial",
    "volume",
];

const _POSSIBLE_ENDPOINT_PROPERTIES: [&str; 8] = [
    "is-default",
    "description",
    "icon",
    "mute",
    "name",
    "path",
    "serial",
    "volume",
];

static NODES: once_cell::sync::OnceCell<Mutex<Vec<ffi::Node>>> = once_cell::sync::OnceCell::new();
static ENDPOINTS: once_cell::sync::OnceCell<Mutex<Vec<ffi::Endpoint>>> = once_cell::sync::OnceCell::new();

pub fn get_node(id: i32) -> Option<ffi::Node> {
    NODES.get().and_then(|nodes| {
        nodes.lock().unwrap().iter().find(|&n| n.id == id).cloned()
    })
}

pub fn get_endpoint(id: i32) -> Option<ffi::Endpoint> {
    ENDPOINTS.get().and_then(|endpoints| {
        endpoints.lock().unwrap().iter().find(|&e| e.node.id == id).cloned()
    })
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
                },

                WpEvent::UpdateEndpoint(id, property_name) => {
                    if let Some(endpoint) = get_endpoint(id) {
                        let _ = ENDPOINTS.get().map(|endpoints| {
                            let mut endpoints = endpoints.lock().unwrap();
                            
                            if let Some(existing_endpoint) = endpoints.iter_mut().find(|e| e.node.id == endpoint.node.id) {
                                match property_name.as_str() {
                                    "is-default" => existing_endpoint.is_default = ffi::endpoint_get_is_default(id),
                                    // Node is Endpoint's ancestor, so updates for the underlying node will be handled
                                    // here as well.
                                    "description" => existing_endpoint.node.description = ffi::node_get_description(id),
                                    "icon" => existing_endpoint.node.icon = ffi::node_get_icon(id),
                                    "mute" => existing_endpoint.node.mute = ffi::node_get_mute(id),
                                    "name" => existing_endpoint.node.name = ffi::node_get_name(id),
                                    "path" => existing_endpoint.node.path = ffi::node_get_path(id),
                                    "serial" => existing_endpoint.node.serial = ffi::node_get_serial(id),
                                    "volume" => existing_endpoint.node.volume = ffi::node_get_volume(id),
                                    _ => {}
                                }
                            }
                        });
                    }
                },

                WpEvent::UpdateDefaultMicrophone(id) => {
                    println!("Default microphone updated: {}", id);
                },

                WpEvent::UpdateDefaultSpeaker(id) => {
                    println!("Default speaker updated: {}", id);
                },

                WpEvent::CreateStream(node) => {
                    let _ = NODES.get().map(|nodes| {
                        nodes.lock().unwrap().push(node);
                    });
                },

                WpEvent::RemoveStream(node) => {
                    let _ = NODES.get().map(|nodes| {
                        nodes.lock().unwrap().retain(|n| n.id != node.id);
                    });
                },

                WpEvent::CreateMicrophone(endpoint) => {
                    let _ = ENDPOINTS.get().map(|endpoints| {
                        endpoints.lock().unwrap().push(endpoint);
                    });
                },

                WpEvent::RemoveMicrophone(endpoint) => {
                    let _ = ENDPOINTS.get().map(|endpoints| {
                        endpoints.lock().unwrap().retain(|e| e.node.id != endpoint.node.id);
                    });
                },

                WpEvent::CreateSpeaker(endpoint) => {
                    let _ = ENDPOINTS.get().map(|endpoints| {
                        endpoints.lock().unwrap().push(endpoint);
                    });
                },

                WpEvent::RemoveSpeaker(endpoint) => {
                    let _ = ENDPOINTS.get().map(|endpoints| {
                        endpoints.lock().unwrap().retain(|e| e.node.id != endpoint.node.id);
                    });
                }
            }

            println!("nodes: {:?}", NODES.get().map(|n| n.lock().unwrap()));
            println!("endpoints: {:?}", ENDPOINTS.get().map(|e| e.lock().unwrap()));
        }
    });

    // Run the WirePlumber main loop in a separate thread.
    std::thread::spawn(ffi::init);
}