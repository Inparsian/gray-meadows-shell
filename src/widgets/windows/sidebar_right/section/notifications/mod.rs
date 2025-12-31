use crate::singletons::notifications;
use crate::singletons::notifications::bus::BusEvent;
use super::SideRightSection;

pub fn new() -> SideRightSection {
    let section = SideRightSection::new(
        "Notifications",
        "notifications",
        "0",
    );
    section.set_toggled(true);

    let mut receiver = notifications::subscribe();
    gtk4::glib::spawn_future_local({
        let section = section.clone();
        async move {
            while let Ok(event) = receiver.recv().await {
                match event {
                    BusEvent::NotificationAdded { .. } |
                    BusEvent::NotificationClosed { .. } => {
                        let count = notifications::NOTIFICATIONS.get()
                            .map_or(0, |notifications| notifications.read().unwrap().len());
                    
                        section.set_content(&count.to_string());
                    },
                
                    _ => {}
                }
            }
        }
    });

    section
}