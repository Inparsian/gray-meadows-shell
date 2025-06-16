fn main() {
    // Build libqalculate FFI bindings
    cxx_build::bridge("src/ffi/libqalculate.rs")
        .file("src/ffi/ext/libqalculate.cc")
        .include("src/ffi/ext")
        .compile("libqalculate");

    println!("cargo:rustc-link-lib=qalculate");
    println!("cargo:rerun-if-changed=src/ffi/ext/libqalculate.h");
    println!("cargo:rerun-if-changed=src/ffi/ext/libqalculate.cc");
    println!("cargo:rerun-if-changed=src/ffi/libqalculate.rs");
}