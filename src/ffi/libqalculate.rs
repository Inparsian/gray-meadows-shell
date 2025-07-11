/*
    Minimal Rust FFI bindings for libqalculate, a C++ library for advanced calculations.
    For this project's current use case, we only need a Calculator object, definition loading,
    and expression unlocalization and calculation. This may be extended in the future.
*/

#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("main.h");

        pub fn init_calc();

        pub fn loadExchangeRates() -> bool;
        pub fn loadGlobalDefinitions() -> bool;
        pub fn loadLocalDefinitions() -> bool;
        pub fn unlocalizeExpression(str: String) -> String;
        pub fn calculateAndPrint(str: String, msecs: u32) -> String;
    }
}