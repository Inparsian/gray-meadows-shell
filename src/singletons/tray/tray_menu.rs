use gdk4::gio::{self, prelude::ActionMapExtManual};
use system_tray::{client::ActivateRequest, menu};

use crate::singletons::tray;

fn label_to_action(label: Option<&str>) -> Option<String> {
    label.map(|l| {
        l.chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>()
            .to_lowercase()
    })
}

fn label_to_prefixed_action(label: Option<&str>) -> Option<String> {
    label.map(|l| format!("dbusmenu.{}", label_to_action(Some(l)).unwrap_or_default()))
}

fn tray_menu_item_to_gio_menu_item(item: menu::MenuItem) -> Option<gio::MenuItem> {
    let menu_item = gio::MenuItem::new(item.label.as_deref(), label_to_prefixed_action(item.label.as_deref()).as_deref());

    Some(menu_item)
}

fn activate_item(owner: String, id: i32) {
    println!("Activating item: {} with id: {}", owner, id);

    if let (Some(client), Some(tray_item)) = (tray::TRAY_CLIENT.get(), tray::get_tray_item(&owner)) {
        let request = ActivateRequest::MenuItem {
            address: owner,
            menu_path: tray_item.menu.unwrap_or_default(),
            submenu_id: id
        };

        tokio::spawn(client.activate(request));
    }
}

pub fn build_gio_tray_menu_model(owner: String) -> Option<(gio::MenuModel, gio::SimpleActionGroup)> {
    if let Some(tray_menu) = tray::get_tray_menu(&owner) {
        let menu = gio::Menu::new();
        let action_group = gio::SimpleActionGroup::new();
        let mut action_entries = vec![];

        tray_menu.submenus.iter().for_each({
            let owner = owner.clone();
            let menu = &menu;
            let action_entries = &mut action_entries;

            move |submenu| {
                // TODO: Empty labels indicate the end of a section.
                if submenu.label.is_none() || submenu.label.as_ref().unwrap().is_empty() {
                    return;
                }

                if !submenu.submenu.is_empty() {
                    // i'm not nesting deeper than this
                    let sub_menu = gio::Menu::new();

                    submenu.submenu.iter().for_each(|item| {
                        let action = gio::ActionEntry::builder(label_to_action(item.label.as_deref()).as_deref().unwrap_or_default())
                            .activate({
                                let owner = owner.clone();
                                let item = item.clone();
                                move |_: &gio::SimpleActionGroup, _, _| {
                                    activate_item(owner.clone(), item.id);
                                }
                            })
                            .build();

                        sub_menu.append_item(&tray_menu_item_to_gio_menu_item(item.clone()).unwrap());
                        action_entries.push(action);
                    });

                    let sub_menu_model: gio::MenuModel = sub_menu.into();

                    menu.insert_submenu(submenu.id, submenu.label.as_deref(), &sub_menu_model);
                } else {
                    let action = gio::ActionEntry::builder(label_to_action(submenu.label.as_deref()).as_deref().unwrap_or_default())
                        .activate({
                            let owner = owner.clone();
                            let item = submenu.clone();
                            move |_: &gio::SimpleActionGroup, _, _| {
                                activate_item(owner.clone(), item.id);
                            }
                        })
                        .build();

                    menu.insert_item(submenu.id, &tray_menu_item_to_gio_menu_item(submenu.clone()).unwrap());
                    action_entries.push(action);
                }
            }
        });

        action_group.add_action_entries(action_entries);

        Some((menu.into(), action_group))
    } else {
        println!("No tray menu found for owner: {}", owner);
        None
    }
}