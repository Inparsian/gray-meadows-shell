#pragma once
#include <libqalculate/qalculate.h>
#include <rust/cxx.h>
#include <memory>

void init_calc();

bool loadExchangeRates();
bool loadGlobalDefinitions();
bool loadLocalDefinitions();

rust::String unlocalizeExpression(
    rust::String str
);

rust::String calculateAndPrint(
    rust::String str,
    rust::u32 msecs
);