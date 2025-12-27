pub enum SessionAction {
    Lock,
    Logout,
    Suspend,
    Hibernate,
    Reboot,
    Shutdown,
}

impl SessionAction {
    pub fn run(&self) {
        let command = match self {
            SessionAction::Lock => "loginctl lock-session",
            SessionAction::Logout => "pkill Hyprland || loginctl terminate-user $USER",
            SessionAction::Suspend => "systemctl suspend || loginctl suspend",
            SessionAction::Hibernate => "systemctl hibernate || loginctl hibernate",
            SessionAction::Reboot => "systemctl reboot || loginctl reboot",
            SessionAction::Shutdown => "systemctl poweroff || loginctl poweroff",
        };

        std::process::Command::new("bash")
            .arg("-c")
            .arg(command)
            .output()
            .expect("Failed to execute command");
    }
}