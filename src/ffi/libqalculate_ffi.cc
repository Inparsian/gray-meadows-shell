#include <libqalculate_ffi.h>
#include <libqalculate/qalculate.h>
#include <rust/cxx.h>
#include <memory>

std::unique_ptr<Calculator> create_calculator() {
    return std::make_unique<Calculator>();
}

bool loadExchangeRates(Calculator &self) {
    return self.loadExchangeRates();
}

bool loadGlobalDefinitions(Calculator &self) {
    return self.loadGlobalDefinitions();
}

bool loadLocalDefinitions(Calculator &self) {
    return self.loadLocalDefinitions();
}

rust::String unlocalizeExpression(Calculator &self, rust::String str) {
    std::string in = str.c_str();
    std::string out = self.unlocalizeExpression(in);
    return rust::String(std::move(out));
}

rust::String calculateAndPrint(Calculator &self, rust::String str, int msecs) {
    std::string in = str.c_str();
    std::string out = self.calculateAndPrint(in, msecs);
    return rust::String(std::move(out));
}