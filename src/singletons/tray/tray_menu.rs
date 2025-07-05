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

fn tray_menu_item_to_gio_menu_item(item: dbus_menu::MenuItem) -> Option<gio::MenuItem> {
    let menu_item = gio::MenuItem::new(Some(&item.label), label_to_prefixed_action(item.label.clone()).as_deref());

    Some(menu_item)
}

pub fn build_gio_tray_menu_model(item: StatusNotifierItem) -> Option<(gio::MenuModel, gio::SimpleActionGroup)> {
    println!("Building GIO tray menu model for service: {}", item.service);

    let tray_menu = item.menu;
    let menu = gio::Menu::new();
    let action_group = gio::SimpleActionGroup::new();
    let mut action_entries = vec![];

    if let Ok(tray_layout) = tray_menu.get_layout() {
        tray_layout.items.iter().for_each({
            let tray_menu = tray_menu.clone();
            let menu = &menu;
            let action_entries = &mut action_entries;

            move |submenu| {
                // TODO: Empty labels indicate the end of a section.
                if submenu.label.is_empty() {
                    return;
                }

                if !submenu.submenus.is_empty() {
                    // i'm not nesting deeper than this
                    let sub_menu = gio::Menu::new();

                    submenu.submenus.iter().for_each(|item| {
                        let action = gio::ActionEntry::builder(label_to_action(item.label.clone()).as_deref().unwrap_or_default())
                            .activate({
                                let tray_menu = tray_menu.clone();
                                let item = item.clone();

                                move |_: &gio::SimpleActionGroup, _, _| {
                                    let _ = tray_menu.activate(item.id);
                                }
                            })
                            .build();

                        sub_menu.append_item(&tray_menu_item_to_gio_menu_item(item.clone()).unwrap());
                        action_entries.push(action);
                    });

                    let sub_menu_model: gio::MenuModel = sub_menu.into();

                    menu.insert_submenu(submenu.id as i32, Some(&submenu.label), &sub_menu_model);
                } else {
                    let action = gio::ActionEntry::builder(label_to_action(submenu.label.clone()).as_deref().unwrap_or_default())
                        .activate({
                            let tray_menu = tray_menu.clone();
                            let item = submenu.clone();

                            move |_: &gio::SimpleActionGroup, _, _| {
                                let _ = tray_menu.activate(item.id);
                            }
                        })
                        .build();

                    menu.insert_item(submenu.id as i32, &tray_menu_item_to_gio_menu_item(submenu.clone()).unwrap());
                    action_entries.push(action);
                }
            }
        });

        action_group.add_action_entries(action_entries);

        println!("Done building GIO tray menu model for service: {}", item.service);

        Some((menu.into(), action_group))
    } else {
        None
    }
}