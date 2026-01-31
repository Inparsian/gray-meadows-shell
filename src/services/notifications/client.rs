use std::collections::HashMap;

use super::proxy::client::OrgFreedesktopNotifications;

#[derive(Default, Clone)]
pub struct NotificationBuilder {
    app_name: Option<String>,
    replaces_id: Option<u32>,
    app_icon: Option<String>,
    summary: Option<String>,
    body: Option<String>,
    actions: Option<Vec<String>>,
    hints: Option<HashMap<String, String>>,
    expire_timeout: Option<i32>,
}

#[allow(dead_code)]
impl NotificationBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn app_name(mut self, app_name: &str) -> Self {
        self.app_name = Some(app_name.to_owned());
        self
    }

    pub fn replaces_id(mut self, replaces_id: u32) -> Self {
        self.replaces_id = Some(replaces_id);
        self
    }

    pub fn app_icon(mut self, app_icon: &str) -> Self {
        self.app_icon = Some(app_icon.to_owned());
        self
    }

    pub fn summary(mut self, summary: &str) -> Self {
        self.summary = Some(summary.to_owned());
        self
    }

    pub fn body(mut self, body: &str) -> Self {
        self.body = Some(body.to_owned());
        self
    }

    pub fn actions(mut self, actions: Vec<(&str, &str)>) -> Self {
        // Flatten the pairs; even elements are identifiers, odd elements are
        // localized names
        let actions = actions
            .into_iter()
            .flat_map(|(key, value)| vec![key.to_owned(), value.to_owned()])
            .collect();
        
        self.actions = Some(actions);
        self
    }

    pub fn hints(mut self, hints: HashMap<String, String>) -> Self {
        self.hints = Some(hints);
        self
    }

    pub fn expire_timeout(mut self, expire_timeout: i32) -> Self {
        self.expire_timeout = Some(expire_timeout);
        self
    }

    pub fn send(self) -> Result<u32, dbus::Error> {
        use dbus::arg::{Variant, PropMap};

        let NotificationBuilder {
            app_name,
            replaces_id,
            app_icon,
            summary,
            body,
            actions,
            hints,
            expire_timeout,
        } = self;

        let connection = dbus::blocking::Connection::new_session()?;
        let proxy = connection.with_proxy(
            "org.freedesktop.Notifications",
            "/org/freedesktop/Notifications",
            std::time::Duration::from_secs(1),
        );

        let app_name = app_name.unwrap_or_else(String::new);
        let app_icon = app_icon.unwrap_or_else(String::new);
        let summary = summary.unwrap_or_else(String::new);
        let body = body.unwrap_or_else(String::new);
        let actions = actions.unwrap_or_else(Vec::new);
        
        let mut hints_map = PropMap::new();
        if let Some(hints) = hints {
            for (key, value) in hints {
                hints_map.insert(key, Variant(Box::new(value)));
            }
        }

        OrgFreedesktopNotifications::notify(
            &proxy,
            app_name.as_str(),
            replaces_id.unwrap_or(0),
            app_icon.as_str(),
            summary.as_str(),
            body.as_str(),
            actions.iter().map(|s| s.as_str()).collect(),
            hints_map,
            expire_timeout.unwrap_or(-1),
        )
    }
}