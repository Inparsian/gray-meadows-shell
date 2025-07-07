use crate::ffi::astalwp::{ffi, CHANNEL, WpEvent};

pub fn activate() {
    // Run the WirePlumber main loop in a separate thread.
    std::thread::spawn(ffi::init);

    let mut receiver = CHANNEL.subscribe();
    tokio::spawn(async move {
        while let Ok(event) = receiver.recv().await {
            match event {
                WpEvent::UpdateNode(node, _property_name) => {
                    println!("Received update for node: {}", node.id);
                },

                WpEvent::UpdateEndpoint(endpoint, _property_name) => {
                    println!("Received update for endpoint: {}", endpoint.node.id);
                },

                WpEvent::CreateStream(node) => {
                    println!("Received create stream for node: {}", node.id);
                },

                WpEvent::RemoveStream(node) => {
                    println!("Received remove stream for node: {}", node.id);
                },

                WpEvent::CreateMicrophone(endpoint) => {
                    println!("Received create microphone for endpoint: {}", endpoint.node.id);
                },

                WpEvent::RemoveMicrophone(endpoint) => {
                    println!("Received remove microphone for endpoint: {}", endpoint.node.id);
                },

                WpEvent::CreateSpeaker(endpoint) => {
                    println!("Received create speaker for endpoint: {}", endpoint.node.id);
                },

                WpEvent::RemoveSpeaker(endpoint) => {
                    println!("Received remove speaker for endpoint: {}", endpoint.node.id);
                }
            }
        }
    });
}