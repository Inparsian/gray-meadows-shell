/*
    Minimal Rust FFI bindings for libqalculate, a C++ library for advanced calculations.
    For this project's current use case, we only need a Calculator object, definition loading,
    and expression unlocalization and calculation. This may be extended in the future.
*/

#![allow(dead_code)] // shut up please

#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("main.h");

        type Calculator;

        fn create_calculator() -> UniquePtr<Calculator>;
        fn loadExchangeRates(calculator: Pin<&mut Calculator>) -> bool;
        fn loadGlobalDefinitions(calculator: Pin<&mut Calculator>) -> bool;
        fn loadLocalDefinitions(calculator: Pin<&mut Calculator>) -> bool;
        fn unlocalizeExpression(calculator: Pin<&mut Calculator>, str: String) -> String;
        fn calculateAndPrint(calculator: Pin<&mut Calculator>, str: String, msecs: i32) -> String;
    }
}

pub struct Calculator {
    calculator: cxx::UniquePtr<ffi::Calculator>,
}

impl Calculator {
    pub fn new() -> Self {
        let calculator = ffi::create_calculator();
        Self { calculator }
    }

    pub fn load_exchange_rates(&mut self) -> bool {
        ffi::loadExchangeRates(self.calculator.pin_mut())
    }

    pub fn load_global_definitions(&mut self) -> bool {
        ffi::loadGlobalDefinitions(self.calculator.pin_mut())
    }

    pub fn load_local_definitions(&mut self) -> bool {
        ffi::loadLocalDefinitions(self.calculator.pin_mut())
    }

    pub fn unlocalize_expression(&mut self, expr: String) -> String {
        ffi::unlocalizeExpression(self.calculator.pin_mut(), expr)
    }

    pub fn calculate_and_print(&mut self, expr: String, msecs: i32) -> String {
        ffi::calculateAndPrint(self.calculator.pin_mut(), expr, msecs)
    }
}