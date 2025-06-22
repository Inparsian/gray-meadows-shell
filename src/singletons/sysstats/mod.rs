use once_cell::sync::Lazy;
use std::sync::Mutex;

pub static SYS: Lazy<Mutex<sysinfo::System>> = Lazy::new(|| {
    Mutex::new(sysinfo::System::new_all())
});