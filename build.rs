fn main() {
    cxx_build::bridge("src/ffi/libqalculate.rs")
        .file("src/ffi/libqalculate_ffi.cc")
        .include("include")
        .compile("libqalculate_ffi");

    // link in the system’s libqalculate
    println!("cargo:rustc-link-lib=qalculate");
    println!("cargo:rerun-if-changed=include/libqalculate_ffi.h");
    println!("cargo:rerun-if-changed=src/ffi/libqalculate_ffi.cc");
    println!("cargo:rerun-if-changed=src/ffi/libqalculate.rs");
}