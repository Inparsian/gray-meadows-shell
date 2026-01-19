mod streams;

use gtk4::prelude::*;

use crate::ffi::astalwp::WpEvent;
use crate::singletons::wireplumber;
use crate::widgets::common::tabs::{Tabs, TabsStack, TabSize};

pub fn new() -> gtk4::Box {
    let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    root.set_css_classes(&["audio-tab-root"]);

    let streams = streams::AudioStreams::default();
    let recorders = streams::AudioStreams::default();
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
                
                _ => {},
            }
        }
    });

    root
}