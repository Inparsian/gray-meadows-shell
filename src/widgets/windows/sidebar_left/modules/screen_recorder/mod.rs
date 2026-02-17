mod audio_targets;

use futures_signals::signal::SignalExt as _;
use gtk::prelude::*;

use crate::config::{read_config, save_config};
use crate::services::screen_recorder::{
    get_screen_recorder,
    ScreenRecorderState,
    ScreenRecorderCaptureOption,
    get_configured_capture_target
};
use self::audio_targets::{AudioTargets, AudioTargetType};

pub fn new() -> gtk::Box {
    let capture_target_model = gio::ListStore::new::<glib::BoxedAnyObject>();
    let capture_target_expression = gtk::ClosureExpression::new::<String>(
        &[] as &[gtk::Expression],
        glib::closure!(|obj: glib::Object| {
            let obj = obj
                .downcast::<glib::BoxedAnyObject>()
                .expect("DropDown model item must be a BoxedAnyObject");

            let borrowed = obj
                .borrow::<(ScreenRecorderCaptureOption, String)>();
            borrowed.1.clone()
        }),
    );

    let capture_target_dropdown = gtk::DropDown::builder()
        .model(&capture_target_model)
        .expression(capture_target_expression.as_ref())
        .selected(0)
        .build();
    
    let capture_target_ignore_next_notify = std::rc::Rc::new(std::cell::Cell::new(true));
    glib::spawn_future_local(clone!(
        #[weak] capture_target_dropdown,
        #[weak] capture_target_model,
        #[strong] capture_target_ignore_next_notify,
        async move {
            let Ok(capture_options) = get_screen_recorder().read()
                .map(|screen_recorder| screen_recorder.capture_options.clone())
            else {
                return;
            };

            for option in capture_options {
                let localized = option.as_localized();
                capture_target_model.append(&glib::BoxedAnyObject::new((option, localized)));
            }

            let configured = get_configured_capture_target();

            capture_target_ignore_next_notify.set(true);
            if let Some((index, _)) = configured {
                capture_target_dropdown.set_selected(index as u32);
            } else {
                // set to portal, which should be the last item
                capture_target_dropdown.set_selected(capture_target_model.n_items().saturating_sub(1));
            }
        }
    ));
    
    capture_target_dropdown.connect_selected_notify(clone!(
        #[weak] capture_target_model,
        move |dropdown| {
            if capture_target_ignore_next_notify.replace(false) {
                return;
            }

            let selected = dropdown.selected();

            if let Some(item) = capture_target_model.item(selected) {
                let obj = item
                    .downcast::<glib::BoxedAnyObject>()
                    .expect("DropDown model item must be a BoxedAnyObject");

                let option = obj
                    .borrow::<(ScreenRecorderCaptureOption, String)>()
                    .clone();

                let mut config = read_config().clone();
                config.screen_recorder.capture_target = option.0.as_config_option();
                let _ = save_config(&config);
            }
        }
    ));
    
    view! {
        start_recording_button = gtk::Button {
            set_css_classes: &["screen-recorder-state-button"],
            set_label: "Record",
            set_hexpand: true,
            connect_clicked: move |_| if let Ok(mut screen_recorder) = get_screen_recorder().write() {
                screen_recorder.start(false);
            }
        },
        
        start_replay_button = gtk::Button {
            set_css_classes: &["screen-recorder-state-button"],
            set_label: "Replay",
            set_hexpand: true,
            connect_clicked: move |_| if let Ok(mut screen_recorder) = get_screen_recorder().write() {
                screen_recorder.start(true);
            }
        },
        
        stop_button = gtk::Button {
            set_css_classes: &["screen-recorder-state-button"],
            set_label: "Stop",
            set_hexpand: true,
            set_sensitive: false,
            connect_clicked: move |_| if let Ok(mut screen_recorder) = get_screen_recorder().write() {
                screen_recorder.stop();
            }
        },
        
        save_button = gtk::Button {
            set_css_classes: &["screen-recorder-state-button"],
            set_label: "Save Replay",
            set_hexpand: true,
            set_sensitive: false,
            connect_clicked: move |_| if let Ok(screen_recorder) = get_screen_recorder().read() {
                screen_recorder.save_replay();
            }
        },
        
        widget = gtk::Box {
            set_css_classes: &["ScreenRecorder"],
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 24,
            set_hexpand: true,
            
            gtk::Box {
                set_css_classes: &["screen-recorder-state-buttons"],
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 8,
                set_hexpand: true,
                
                append: &start_recording_button,
                append: &start_replay_button,
                append: &stop_button,
                append: &save_button,
            },
            
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 8,
                set_hexpand: true,
                
                gtk::Label {
                    set_css_classes: &["screen-recorder-field-label"],
                    set_label: "Capture Target",
                    set_xalign: 0.0,
                    set_hexpand: true,
                },
                append: &capture_target_dropdown,
            },
            
            append: &AudioTargets::new(AudioTargetType::App).root,
            append: &AudioTargets::new(AudioTargetType::Device).root,
        }
    };
    
    if let Ok(screen_recorder) = get_screen_recorder().read() {
        glib::spawn_future_local(signal!(screen_recorder.state, (new_state) {
            let active = matches!(new_state, ScreenRecorderState::Record | ScreenRecorderState::Replay);
            let idle = matches!(new_state, ScreenRecorderState::Idle);
            
            start_recording_button.set_sensitive(idle);
            start_replay_button.set_sensitive(idle);
            stop_button.set_sensitive(active);
            save_button.set_sensitive(new_state == ScreenRecorderState::Replay);
        }));
    }

    widget
}