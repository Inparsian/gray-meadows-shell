use dbus::Message;

#[derive(Debug, Clone, Copy)]
pub struct MprisPlayer {
    pub bus: internment::Intern<String>,
    pub owner: internment::Intern<String>,
    pub time: u16
}

impl MprisPlayer {
    pub fn new(bus: String, owner: String) -> Self {
        MprisPlayer {
            bus: internment::Intern::new(bus),
            owner: internment::Intern::new(owner),
            time: 0
        }
    }

    pub fn properties_changed(&mut self, msg: &Message) {
        println!("[{}] {}::PropertiesChanged", self.time, self.bus);

        self.time += 1;
    }

    pub fn seeked(&mut self, msg: &Message) {
        println!("[{}] {}::Seeked", self.time, self.bus);

        self.time += 1;
    }
}