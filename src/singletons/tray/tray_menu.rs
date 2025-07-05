use gdk4::gio::{self, prelude::ActionMapExtManual};

use crate::singletons::tray::wrapper::{dbus_menu, sn_item::StatusNotifierItem};

fn label_to_action(label: String) -> Option<String> {
    Some(label.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>()
        .to_lowercase())
}

fn label_to_prefixed_action(label: String) -> Option<String> {
    Some(format!("dbusmenu.{}", label_to_action(label).unwrap_or_default()))
}

fn dbus_menu_item_to_gio_menu_item(item: dbus_menu::MenuItem) -> Option<gio::MenuItem> {
    let menu_item = gio::MenuItem::new(Some(&item.label), label_to_prefixed_action(item.label.clone()).as_deref());

    Some(menu_item)
}

fn build_gio_tray_submenu_model(
    dbus_menu: &dbus_menu::DbusMenu,
    item: dbus_menu::MenuItem,
    action_entries: &mut Vec<gio::ActionEntry<gio::SimpleActionGroup>>,
) -> Option<gio::MenuModel> {
    let sub_menu = gio::Menu::new();

    item.submenus.iter().for_each(|item| {
        let action = gio::ActionEntry::builder(label_to_action(item.label.clone()).as_deref().unwrap_or_default())
            .activate({
                let dbus_menu = dbus_menu.clone();
                let item = item.clone();

                move |_: &gio::SimpleActionGroup, _, _| dbus_menu.activate(item.id).unwrap_or(())
            })
            .build();

        if !item.submenus.is_empty() {
            let sub_menu_model = build_gio_tray_submenu_model(dbus_menu, item.clone(), action_entries).unwrap();

            sub_menu.append_submenu(Some(&item.label), &sub_menu_model);
        } else {
            let menu_item = dbus_menu_item_to_gio_menu_item(item.clone()).unwrap();
            
            sub_menu.append_item(&menu_item);
        }
        
        action_entries.push(action);
    });

    Some(sub_menu.into())
}

pub fn build_gio_dbus_menu_model(item: StatusNotifierItem) -> Option<(gio::MenuModel, gio::SimpleActionGroup)> {
    let dbus_menu = item.menu;
    let menu = gio::Menu::new();
    let action_group = gio::SimpleActionGroup::new();
    let mut action_entries = vec![];

    if let Ok(menu_layout) = dbus_menu.get_layout() {
        for item in menu_layout.items.iter() {
            if item.type_ == "separator" {
                continue;
            }

            if !item.submenus.is_empty() {
                let sub_menu_model = build_gio_tray_submenu_model(&dbus_menu, item.clone(), &mut action_entries).unwrap();

                menu.insert_submenu(item.id as i32, Some(&item.label), &sub_menu_model);
            } else {
                let action = gio::ActionEntry::builder(label_to_action(item.label.clone()).as_deref().unwrap_or_default())
                    .activate({
                        let dbus_menu = dbus_menu.clone();
                        let item = item.clone();

                        move |_: &gio::SimpleActionGroup, _, _| dbus_menu.activate(item.id).unwrap_or(())
                    })
                    .build();

                menu.insert_item(item.id as i32, &dbus_menu_item_to_gio_menu_item(item.clone()).unwrap());
                action_entries.push(action);
            }
        }

        action_group.add_action_entries(action_entries);

        Some((menu.into(), action_group))
    } else {
        None
    }
}