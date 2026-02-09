use gtk::prelude::*;

use crate::config::{read_config, save_config};
use crate::services::screen_recorder::{SCREEN_RECORDER, ScreenRecorderCaptureOption, get_configured_capture_target};

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
            let Some(capture_options) = SCREEN_RECORDER.get()
                .map(|screen_recorder| screen_recorder.read().unwrap().capture_options.clone())
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
                capture_target_dropdown.set_selected(capture_target_model.n_items() - 1);
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
        widget = gtk::Box {
            set_css_classes: &["ScreenRecorder"],
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 8,
            set_hexpand: true,
            
            append: &capture_target_dropdown,
        }
    };

    widget
}