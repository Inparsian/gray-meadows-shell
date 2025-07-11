use crate::ffi::libqalculate::ffi;

pub fn activate() {
    ffi::init_calc();
    ffi::loadExchangeRates();
    ffi::loadGlobalDefinitions();
    ffi::loadLocalDefinitions();
}