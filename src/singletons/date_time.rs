use futures_signals::signal::Mutable;
use once_cell::sync::Lazy;
use chrono::Local;

const DATE_FORMAT: &str = "%a, %m/%d";
const TIME_FORMAT: &str = "%I:%M %p";

#[derive(Clone, Debug)]
pub struct DateTime {
    pub date: String,
    pub time: String
}

pub static DATE_TIME: Lazy<Mutable<DateTime>> = Lazy::new(|| Mutable::new(date_time_now()));

fn date_time_now() -> DateTime {
    let now = Local::now();

    DateTime {
        date: now.format(DATE_FORMAT).to_string(),
        time: now.format(TIME_FORMAT).to_string(),
    }
}

pub fn activate() {
    tokio::spawn(async move {
        loop {
            DATE_TIME.set(date_time_now());
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    });
}