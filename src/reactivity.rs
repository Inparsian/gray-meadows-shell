use futures_signals::signal::{Mutable, SignalExt};

// Macros to create signals for a Mutable variable and subscribe to it,
// allowing a widget to effectively react to changes in the Mutable's value.
#[macro_export]
macro_rules! subscribe {
    ($mutable:expr, $widget:expr, $method:ident) => {
        let future = $mutable.signal().for_each(move |value| {
            $widget.$method(&value);
            async {}
        });

        gtk4::glib::MainContext::default().spawn_local(future);
    };
}

#[macro_export]
macro_rules! subscribeCloned {
    ($mutable:expr, $widget:expr, $method:ident) => {
        let future = $mutable.signal_cloned().for_each(move |value| {
            $widget.$method(&value);
            async {}
        });

        gtk4::glib::MainContext::default().spawn_local(future);
    };
}

// Helper functions to make writing reactive GTK widgets easier w/ Relm4 syntax
pub fn reactive_label(mutable: &Mutable<String>) -> gtk4::Label {
    let widget = gtk4::Label::new(Some("..."));
    let widget_clone = widget.clone();
    subscribeCloned!(mutable, widget_clone, set_label);
    widget
}