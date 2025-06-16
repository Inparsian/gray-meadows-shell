use futures_signals::signal::Mutable;
use once_cell::sync::Lazy;

const DATE_FORMAT: &str = "%a, %m/%d";
const TIME_FORMAT: &str = "%I:%M %p";

pub struct DateTime {
    pub date: futures_signals::signal::Mutable<String>,
    pub time: futures_signals::signal::Mutable<String>,
}

pub static DATE_TIME: Lazy<DateTime> = Lazy::new(|| {
    DateTime {
        date: Mutable::new(chrono::Local::now().format(DATE_FORMAT).to_string()),
        time: Mutable::new(chrono::Local::now().format(TIME_FORMAT).to_string()),
    }
});

fn update_datetime() {
    let now = chrono::Local::now();
    let date = now.format(DATE_FORMAT).to_string();
    let time = now.format(TIME_FORMAT).to_string();

    DATE_TIME.date.set(date);
    DATE_TIME.time.set(time);
}

pub fn activate() {
    let future = async move {
        loop {
            update_datetime();
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    };

    tokio::spawn(future);
}