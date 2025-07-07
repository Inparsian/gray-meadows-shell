#pragma once
#include <libqalculate/qalculate.h>
#include <rust/cxx.h>
#include <memory>

// factory for Calculator
std::unique_ptr<Calculator> create_calculator();

// Calculator methods
bool loadExchangeRates(Calculator &self);
bool loadGlobalDefinitions(Calculator &self);
bool loadLocalDefinitions(Calculator &self);

rust::string unlocalizeExpression(
    Calculator &self,
    rust::string str
);

rust::string calculateAndPrint(
    Calculator &self,
    rust::string str,
    int msecs
);