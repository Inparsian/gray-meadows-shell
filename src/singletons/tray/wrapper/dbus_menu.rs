use std::time::SystemTime;
use dbus::{arg::{self, RefArg, Variant}, blocking};

use crate::singletons::tray::{bus::{self, make_key_value_pairs}, proxy::menu::RawLayout};

#[derive(Default, Debug, Clone)]
pub struct Menu {
    pub _id: u32,
    pub items: Vec<MenuItem>,
}

#[derive(Default, Debug, Clone)]
pub struct MenuItem {
    pub id: u32,
    pub label: String,
    pub enabled: bool,
    pub children_display: String,
    pub type_: String, // e.g., "normal", "separator", "submenu"
    pub visible: bool,
    pub icon_data: Option<Vec<u8>>,
    pub submenus: Vec<MenuItem>,
}

impl Menu {
    fn unpack_item(struct_: &dyn RefArg) -> Option<MenuItem> {
        let mut item = MenuItem::default();
        let mut iter = struct_.as_iter().unwrap();

        if let (Some(id), Some(properties), Some(submenus)) = (iter.next(), iter.next(), iter.next()) {
            let submenus = submenus.as_iter().unwrap().collect::<Vec<&dyn RefArg>>();
            let properties = make_key_value_pairs(properties);
            
            item.id = id.as_i64().unwrap_or_default() as u32;

            // Iterate through the properties map
            for (key, value) in properties {
                match key.as_str() {
                    "label" => item.label = value.as_str().unwrap_or_default().to_owned(),
                    "enabled" => item.enabled = value.as_i64().unwrap_or(0) != 0,
                    "type" => item.type_ = value.as_str().unwrap_or_default().to_owned(),
                    "visible" => item.visible = value.as_i64().unwrap_or(0) != 0,
                    "children-display" => item.children_display = value.as_str().unwrap_or_default().to_owned(),
                    "icon-data" => if let Some(icon_data) = value.as_iter().and_then(|mut v| v.next().and_then(|v| v.as_iter())) {
                        item.icon_data = icon_data.map(|v| v.as_i64().unwrap_or_default() as u8)
                            .collect::<Vec<u8>>()
                            .into();
                    } else {
                        eprintln!("Icon data is not in the expected format.");
                    },

                    _ => {
                        eprintln!("Unexpected property: {} with value: {:?}", key, value);
                    }
                }
            }

            // Iterate through the submenus
            if !submenus.is_empty() {
                for submenu in submenus {
                    if submenu.arg_type() == arg::ArgType::Variant {
                        let variant = submenu.as_iter().unwrap().next().unwrap();

                        if variant.arg_type() == arg::ArgType::Struct {
                            let subitem = Self::unpack_item(&variant);

                            if let Some(subitem) = subitem {
                                item.submenus.push(subitem);
                            } else {
                                eprintln!("Failed to unpack submenu item.");
                            }
                        } else {
                            eprintln!("Unexpected variant type: {:?}", variant.arg_type());
                        }
                    } else {
                        eprintln!("Unexpected submenu type: {:?}", submenu.arg_type());
                    }
                }
            }

            Some(item)
        } else {
            eprintln!("Failed to unpack struct variant: {:?}", struct_);

            None
        }
    }

    pub fn from_raw(value: RawLayout) -> Self {
        let mut menu = Menu {
            _id: value.0,
            items: Vec::new()
        };

        let bx = &value.1.2;
        let iter = bx.as_iter().unwrap();

        for value in iter {
            let variant = value.as_iter().unwrap().next().unwrap();

            if variant.arg_type() == arg::ArgType::Struct {
                if let Some(item) = Self::unpack_item(&variant) {
                    menu.items.push(item);
                }
            }
        }

        menu
    }
}

#[derive(Default, Debug, Clone)]
pub struct DbusMenu {
    pub service: String,
    pub object: String, // The object path depends on the application's implementation of dbusmenu.
}

impl DbusMenu {
    /// Creates a new `DbusMenu` with the given object path.
    pub fn new(service: String, object: String) -> Self {
        DbusMenu {
            service,
            object
        }
    }

    /// Activates an item on this menu.
    pub fn activate(&self, item_id: u32) -> Result<(), dbus::Error> {
        let connection = blocking::Connection::new_session()?;
        let proxy = connection.with_proxy(
            self.service.clone(),
            self.object.clone(),
            std::time::Duration::from_millis(5000),
        );

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Meow, time is broken");

        // Call the ActivateItem method with the item ID
        let _: Result<(), dbus::Error> = proxy.method_call(bus::DBUSMENU_BUS, "Event", (
            item_id as i32,
            "clicked",
            Variant(0_u32),
            timestamp.as_millis() as u32,
        ));

        Ok(())
    }

    /// Gets the layout of the menu.
    pub fn get_layout(&self) -> Result<Menu, dbus::Error> {
        let connection = blocking::Connection::new_session()?;
        let proxy = connection.with_proxy(
            self.service.clone(),
            self.object.clone(),
            std::time::Duration::from_millis(5000),
        );
        
        let result = proxy.method_call(bus::DBUSMENU_BUS, "GetLayout", (0, 10, Vec::<String>::new(),));

        if let Ok(layout) = result {
            Ok(Menu::from_raw(layout))
        } else {
            eprintln!("Failed to get menu layout: {:?}", result);
            Err(dbus::Error::new_failed("Failed to get menu layout"))
        }
    }
}