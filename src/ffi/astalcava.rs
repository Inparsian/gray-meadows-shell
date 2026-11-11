#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("main.h");

        pub fn init_cava();
        pub fn get_values() -> Vec<f64>;
    }
}
