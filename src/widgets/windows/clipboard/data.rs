mod imp {
    use std::cell::Cell;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    
    #[derive(glib::Properties, Default)]
    #[properties(wrapper_type = super::ClipboardEntryData)]
    pub struct ClipboardEntryData {
        #[property(get, set)]
        id: Cell<i32>,
    }
    
    #[glib::object_subclass]
    impl ObjectSubclass for ClipboardEntryData {
        const NAME: &'static str = "ClipboardEntryData";
        type Type = super::ClipboardEntryData;
    }
    
    #[glib::derived_properties]
    impl ObjectImpl for ClipboardEntryData {}
}

glib::wrapper! {
    pub struct ClipboardEntryData(ObjectSubclass<imp::ClipboardEntryData>);
}

impl ClipboardEntryData {
    pub fn new(id: i32) -> Self {
        glib::Object::builder()
            .property("id", id)
            .build()
    }
}
