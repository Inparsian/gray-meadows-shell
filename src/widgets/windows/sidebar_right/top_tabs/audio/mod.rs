mod master;
mod streams;

use gtk4::prelude::*;

use crate::ffi::astalwp::{WpEvent, ffi::EndpointType};
use crate::services::wireplumber;
use crate::widgets::common::tabs::{Tabs, TabSize};

pub fn new() -> gtk4::Box {
    let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    root.set_css_classes(&["audio-tab-root"]);

    let streams = streams::AudioStreams::default();
    let recorders = streams::AudioStreams::default();
    let mic_master = master::MasterControls::new(EndpointType::Microphone);
    let speaker_master = master::MasterControls::new(EndpointType::Speaker);
    let tabs = Tabs::new(TabSize::Tiny, false, Some("audio-tab-tabs"));
    tabs.add_tab("playback", "playback", None, &streams.root);
    tabs.add_tab("recording", "recording", None, &recorders.root);
    tabs.current_tab.set(Some("playback".to_owned()));
    
    root.append(&tabs.group().vexpand(true).build());
    root.append(&mic_master.root);
    root.append(&speaker_master.root);
    
    let mut channel = wireplumber::subscribe();
    glib::spawn_future_local(async move {
        while let Ok(event) = channel.recv().await {
            match event {
                WpEvent::CreateStream(node) => streams.add_stream(node),
                WpEvent::CreateRecorder(node) => recorders.add_stream(node),
                WpEvent::RemoveStream(node) => streams.remove_stream(&node),
                WpEvent::RemoveRecorder(node) => recorders.remove_stream(&node),
                WpEvent::UpdateNode(id, _) => {
                    streams.update_stream(id);
                    recorders.update_stream(id);
                },
                
                WpEvent::CreateMicrophone(endpoint) => {
                    mic_master.add_device(endpoint.clone());
                    mic_master.update_from(&endpoint, true);
                },
                
                WpEvent::CreateSpeaker(endpoint) => {
                    speaker_master.add_device(endpoint.clone());
                    speaker_master.update_from(&endpoint, true);
                },
                
                WpEvent::RemoveMicrophone(endpoint) => {
                    mic_master.remove_device(&endpoint);
                    if let Some(default_microphone) = wireplumber::get_default_microphone()
                        && default_microphone.node.id == endpoint.node.id
                    {
                        mic_master.update_from(&default_microphone, true);
                    }
                },
            
                WpEvent::RemoveSpeaker(endpoint) => {
                    speaker_master.remove_device(&endpoint);
                    if let Some(default_speaker) = wireplumber::get_default_speaker()
                        && default_speaker.node.id == endpoint.node.id
                    {
                        speaker_master.update_from(&default_speaker, true);
                    }
                },
                
                WpEvent::UpdateDefaultMicrophone(id) => if let Some(microphone) = wireplumber::get_endpoint(id) {
                    mic_master.update_from(&microphone, true);
                },
            
                WpEvent::UpdateDefaultSpeaker(id) => if let Some(speaker) = wireplumber::get_endpoint(id) {
                    speaker_master.update_from(&speaker, true);
                },
            
                WpEvent::UpdateEndpoint(id, property_name) => if property_name == "volume" {
                    if let Some(default_microphone) = wireplumber::get_default_microphone()
                        && default_microphone.node.id == id
                    {
                        mic_master.update_from(&default_microphone, false);
                    }
                    
                    if let Some(default_speaker) = wireplumber::get_default_speaker()
                        && default_speaker.node.id == id
                    {
                        speaker_master.update_from(&default_speaker, false);
                    }
                },
            }
        }
    });

    root
}