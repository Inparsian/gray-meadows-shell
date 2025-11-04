use dbus::{arg, blocking, Path};

use crate::singletons::tray::{icon::compress_icon_pixmap, bus, proxy::item::{RawPixmap, RawToolTip}};

#[allow(dead_code)]
#[derive(Default, Clone, Debug)]
pub struct ToolTip {
    pub icon_name: String,
    pub icon_pixmap: Vec<RawPixmap>,
    pub title: String,
    pub description: String
}

impl ToolTip {
    /// Creates a new `ToolTip` from a raw tooltip (tuple) given by the D-Bus service.
    pub fn from_tuple(tuple: RawToolTip) -> Self {
        // compress the pixmap!!!
        let icon_pixmap = compress_icon_pixmap(Some(&tuple.1));

        ToolTip {
            icon_name: tuple.0,
            icon_pixmap: icon_pixmap.unwrap_or_default(),
            title: tuple.2,
            description: tuple.3
        }
    }
}

/// https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/StatusNotifierItem/
#[derive(Default, Clone, Debug)]
pub struct StatusNotifierItem {
    pub service: String,

    // Properties to store in memory so we don't have to query D-Bus every time.
    pub category: String,
    pub id: String,
    pub title: String,
    pub status: String,
    pub icon_name: String,
    pub icon_pixmap: Vec<RawPixmap>,
    pub overlay_icon_name: String,
    pub overlay_icon_pixmap: Vec<RawPixmap>,
    pub attention_icon_name: String,
    pub attention_icon_pixmap: Vec<RawPixmap>,
    pub attention_movie_name: String,
    pub tool_tip: ToolTip,
    pub item_is_menu: bool,
    pub menu: super::dbus_menu::DbusMenu
}

impl StatusNotifierItem {
    /// Creates a new `StatusNotifierItem` with the given service name.
    pub fn new(service: String) -> Self {
        let mut item = StatusNotifierItem {
            service,
            ..Self::default()
        };

        let _ = item.sync();

        item
    }

    /// Performs synchronization of the StatusNotifierItem with the D-Bus service.
    /// Ideally, this should only be called once after the item is created.
    fn sync(&mut self) -> Result<(), dbus::MethodErr> {
        macro_rules! try_set_prop {
            ($type:ty, $prop:expr, $field:ident) => {
                match self.try_get_prop::<$type>($prop) {
                    Ok(value) => self.$field = value,
                    Err(err) => {
                        eprintln!("Failed to get {}: {}", $prop, err);
                    }
                }
            };

            (pixmap - $prop:expr, $field:ident) => {
                match self.try_get_pixmap($prop) {
                    Ok(pixmap) => self.$field = pixmap,
                    Err(err) => {
                        eprintln!("Failed to get {}: {}", $prop, err);
                    }
                }
            };
        }

        try_set_prop!(String, "Category", category);
        try_set_prop!(String, "Id", id);
        try_set_prop!(String, "Title", title);
        try_set_prop!(String, "Status", status);
        try_set_prop!(String, "IconName", icon_name);
        try_set_prop!(pixmap - "IconPixmap", icon_pixmap);
        try_set_prop!(String, "OverlayIconName", overlay_icon_name);
        try_set_prop!(pixmap - "OverlayIconPixmap", overlay_icon_pixmap);
        try_set_prop!(String, "AttentionIconName", attention_icon_name);
        try_set_prop!(pixmap - "AttentionIconPixmap", attention_icon_pixmap);
        try_set_prop!(String, "AttentionMovieName", attention_movie_name);
        try_set_prop!(bool, "ItemIsMenu", item_is_menu);

        // tooltip
        match self.try_get_prop::<RawToolTip>("ToolTip") {
            Ok(tool_tip) => self.tool_tip = ToolTip::from_tuple(tool_tip),
            Err(err) => eprintln!("Failed to get ToolTip property: {}", err),
        }

        // menu
        match self.try_get_prop::<Path>("Menu") {
            Ok(menu) => self.menu = super::dbus_menu::DbusMenu::new(self.service.clone(), menu.to_string()),
            Err(err) => eprintln!("Failed to get Menu property: {}", err),
        }

        Ok(())
    }

    /// Passes an update to the StatusNotifierItem, updating its properties.
    pub fn pass_update(&mut self, member: &str) {
        match member {
            "NewTitle" => {
                if let Ok(title) = self.try_get_prop::<String>("Title") {
                    self.title = title;
                }
            },

            "NewIcon" => {
                if let Ok(icon_name) = self.try_get_prop::<String>("IconName") {
                    self.icon_name = icon_name;
                }

                if let Ok(icon_pixmap) = self.try_get_pixmap("IconPixmap") {
                    self.icon_pixmap = icon_pixmap;
                }
            },

            "NewAttentionIcon" => {
                if let Ok(attention_icon_name) = self.try_get_prop::<String>("AttentionIconName") {
                    self.attention_icon_name = attention_icon_name;
                }

                if let Ok(attention_icon_pixmap) = self.try_get_pixmap("AttentionIconPixmap") {
                    self.attention_icon_pixmap = attention_icon_pixmap;
                }
            },

            "NewOverlayIcon" => {
                if let Ok(overlay_icon_name) = self.try_get_prop::<String>("OverlayIconName") {
                    self.overlay_icon_name = overlay_icon_name;
                }

                if let Ok(overlay_icon_pixmap) = self.try_get_pixmap("OverlayIconPixmap") {
                    self.overlay_icon_pixmap = overlay_icon_pixmap;
                }
            },

            "NewToolTip" => {
                if let Ok(tool_tip) = self.try_get_prop::<RawToolTip>("ToolTip") {
                    self.tool_tip = ToolTip::from_tuple(tool_tip);
                }
            },

            "NewStatus" => {
                if let Ok(status) = self.try_get_prop::<String>("Status") {
                    self.status = status;
                }
            },

            _ => {}
        }
    }

    /// Attempts to get a RawPixmap from the StatusNotifierItem D-Bus object.
    /// 
    /// Prefer usage of this over `try_get_prop`, since this will automatically
    /// compress the pixmap to a smaller size if it is larger than 32x32.
    pub fn try_get_pixmap(&self, prop: &str) -> Result<Vec<RawPixmap>, dbus::MethodErr> {
        let icon = self.try_get_prop::<Vec<RawPixmap>>(prop)?;
        
        compress_icon_pixmap(Some(&icon))
            .ok_or_else(|| dbus::MethodErr::failed("Failed to compress icon pixmap"))
    }

    /// Attempts to get a property from the StatusNotifierItem D-Bus object with
    /// the given type `T`.
    /// 
    /// If there's a method for your type already, it is highly
    /// recommended to use that instead of this method, since this will return the
    /// property as is, without any compression or processing.
    pub fn try_get_prop<T>(&self, prop: &str) -> Result<T, dbus::MethodErr>
    where
        T: for<'b> arg::Get<'b> + 'static,
    {
        use blocking::stdintf::org_freedesktop_dbus::Properties;

        let connection = blocking::Connection::new_session()?;
        let proxy = connection.with_proxy(
            self.service.clone(),
            bus::ITEM_DBUS_OBJECT,
            std::time::Duration::from_millis(5000),
        );
        
        proxy.get::<T>(bus::ITEM_DBUS_BUS, prop)
            .map_err(|err| dbus::MethodErr::failed(&err.message().unwrap_or_default()))
    }

    /// Sends a primary activation request to the StatusNotifierItem.
    pub fn activate(&self, x: i32, y: i32) -> Result<(), dbus::Error> {
        let connection = blocking::Connection::new_session()?;
        let proxy = connection.with_proxy(
            self.service.clone(),
            bus::ITEM_DBUS_OBJECT,
            std::time::Duration::from_millis(5000),
        );

        // Call the Activate method
        let _: Result<(), dbus::Error> = proxy.method_call(bus::ITEM_DBUS_BUS, "Activate", (x, y,));
        
        Ok(())
    }
}