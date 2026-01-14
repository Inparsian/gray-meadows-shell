use gdk4::gio::{self, prelude::ActionMapExtManual as _};

use super::wrapper::{dbus_menu::{self, Menu}, sn_item::StatusNotifierItem};

fn label_to_action(label: &str) -> Option<String> {
    Some(label.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>()
        .to_lowercase())
}

fn label_to_prefixed_action(label: &str) -> Option<String> {
    Some(format!("dbusmenu.{}", label_to_action(label).unwrap_or_default()))
}

fn dbus_menu_item_to_gio_menu_item(item: &dbus_menu::MenuItem) -> Option<gio::MenuItem> {
    let menu_item = gio::MenuItem::new(Some(&item.label), label_to_prefixed_action(&item.label).as_deref());

    Some(menu_item)
}

fn build_gio_dbus_submenu_model(
    dbus_menu: &dbus_menu::DbusMenu,
    item: &dbus_menu::MenuItem,
    action_entries: &mut Vec<gio::ActionEntry<gio::SimpleActionGroup>>,
) -> Option<gio::MenuModel> {
    let sub_menu = gio::Menu::new();

    item.submenus.iter().for_each(|item| {
        let action = gio::ActionEntry::builder(label_to_action(&item.label).as_deref().unwrap_or_default())
            .activate({
                let dbus_menu = dbus_menu.clone();
                let item = item.clone();

                move |_: &gio::SimpleActionGroup, _, _| if dbus_menu.activate(item.id).is_err() {
                    error!(label = %item.label, "Failed to activate menu item");
                }
            })
            .build();

        if !item.submenus.is_empty() {
            let sub_menu_model = build_gio_dbus_submenu_model(dbus_menu, item, action_entries).unwrap();

            sub_menu.append_submenu(Some(&item.label), &sub_menu_model);
        } else {
            let menu_item = dbus_menu_item_to_gio_menu_item(item).unwrap();
            
            sub_menu.append_item(&menu_item);
        }
        
        action_entries.push(action);
    });

    Some(sub_menu.into())
}

pub fn build_gio_dbus_menu_model_with_layout(item: StatusNotifierItem, menu_layout: &Menu) -> Option<(gio::MenuModel, gio::SimpleActionGroup)> {
    let dbus_menu = item.menu;
    let menu = gio::Menu::new();
    let action_group = gio::SimpleActionGroup::new();
    let mut action_entries = vec![];

    for item in &menu_layout.items {
        if item.type_ == "separator" {
            continue;
        }

        if !item.submenus.is_empty() {
            let sub_menu_model = build_gio_dbus_submenu_model(&dbus_menu, item, &mut action_entries).unwrap();

            menu.insert_submenu(item.id as i32, Some(&item.label), &sub_menu_model);
        } else {
            let action = gio::ActionEntry::builder(label_to_action(&item.label).as_deref().unwrap_or_default())
                .activate({
                    let dbus_menu = dbus_menu.clone();
                    let item = item.clone();

                    move |_: &gio::SimpleActionGroup, _, _| if dbus_menu.activate(item.id).is_err() {
                        error!(label = %item.label, "Failed to activate menu item");
                    }
                })
                .build();

            menu.insert_item(item.id as i32, &dbus_menu_item_to_gio_menu_item(item).unwrap());
            action_entries.push(action);
        }
    }

    action_group.add_action_entries(action_entries);

    Some((menu.into(), action_group))
}