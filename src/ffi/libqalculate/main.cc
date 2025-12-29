#include "main.h"
#include <libqalculate/qalculate.h>
#include <rust/cxx.h>
#include <memory>

static void assert_calculator() {
    if (!calculator) {
        throw std::runtime_error("Calculator instance not initialized");
    }
}

void init_calc() {
    if (!calculator) {
        calculator = new Calculator();
    }
}

bool loadExchangeRates() {
    assert_calculator();
    return calculator->loadExchangeRates();
}

bool loadGlobalDefinitions() {
    assert_calculator();
    return calculator->loadGlobalDefinitions();
}

bool loadLocalDefinitions() {
    assert_calculator();
    return calculator->loadLocalDefinitions();
}

rust::String unlocalizeExpression(rust::String str) {
    assert_calculator();
    std::string in = str.c_str();
    std::string out = calculator->unlocalizeExpression(in);
    return rust::String(std::move(out));
}

rust::String calculateAndPrint(rust::String str, rust::u32 msecs) {
    assert_calculator();
    std::string in = str.c_str();
    std::string out = calculator->calculateAndPrint(in, msecs);
    return rust::String(std::move(out));
}