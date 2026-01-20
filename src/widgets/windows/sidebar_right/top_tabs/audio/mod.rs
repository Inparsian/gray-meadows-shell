mod master;
mod streams;

use gtk4::prelude::*;

use crate::ffi::astalwp::{WpEvent, ffi::EndpointType};
use crate::singletons::wireplumber;
use crate::widgets::common::tabs::{Tabs, TabsStack, TabSize};

pub fn new() -> gtk4::Box {
    let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    root.set_css_classes(&["audio-tab-root"]);

    let streams = streams::AudioStreams::default();
    let recorders = streams::AudioStreams::default();
    let mic_master = master::MasterControls::new(EndpointType::Microphone);
    let speaker_master = master::MasterControls::new(EndpointType::Speaker);
    let tabs = Tabs::new(TabSize::Tiny, false);
    let tabs_stack = TabsStack::new(&tabs, Some("audio-tab-tabs"));
    
    tabs.add_tab(
        "playback",
        "playback".to_owned(),
        None,
    );

    tabs_stack.add_tab(
        Some("playback"),
        &streams.root,
    );

    tabs.add_tab(
        "recording",
        "recording".to_owned(),
        None,
    );

    tabs_stack.add_tab(
        Some("recording"),
        &recorders.root,
    );
    
    tabs.current_tab.set(Some("playback".to_owned()));
    
    root.append(&tabs.widget);
    root.append(&tabs_stack.widget);
    root.append(&mic_master.root);
    root.append(&speaker_master.root);
    
    let mut channel = wireplumber::subscribe();
    gtk4::glib::spawn_future_local(async move {
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
                
                WpEvent::CreateMicrophone(endpoint) => if endpoint.is_default {
                    mic_master.update_from(&endpoint);
                },
                
                WpEvent::CreateSpeaker(endpoint) => if endpoint.is_default {
                    speaker_master.update_from(&endpoint);
                },
                
                WpEvent::RemoveMicrophone(endpoint) => if let Some(default_microphone) = wireplumber::get_default_microphone()
                    && default_microphone.node.id == endpoint.node.id
                {
                    mic_master.update_from(&default_microphone);
                },
            
                WpEvent::RemoveSpeaker(endpoint) => if let Some(default_speaker) = wireplumber::get_default_speaker()
                    && default_speaker.node.id == endpoint.node.id
                {
                    speaker_master.update_from(&default_speaker);
                },
                
                WpEvent::UpdateDefaultMicrophone(id) => if let Some(microphone) = wireplumber::get_endpoint(id) {
                    mic_master.update_from(&microphone);
                },
            
                WpEvent::UpdateDefaultSpeaker(id) => if let Some(speaker) = wireplumber::get_endpoint(id) {
                    speaker_master.update_from(&speaker);
                },
            
                WpEvent::UpdateEndpoint(id, property_name) => if property_name == "volume" {
                    if let Some(default_microphone) = wireplumber::get_default_microphone()
                        && default_microphone.node.id == id
                    {
                        mic_master.update_from(&default_microphone);
                    }
                    
                    if let Some(default_speaker) = wireplumber::get_default_speaker()
                        && default_speaker.node.id == id
                    {
                        speaker_master.update_from(&default_speaker);
                    }
                },
            }
        }
    });

    root
}