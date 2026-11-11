#pragma once
#include <rust/cxx.h>
#include <gray-meadows-shell/src/ffi/astalcava.rs.h>

void init_cava();
rust::Vec<double> get_values();
