fn main() {
    // Build FFI bindings for libqalculate
    cxx_build::bridge("src/ffi/libqalculate.rs")
        .file("src/ffi/ext/libqalculate/main.cc")
        .include("src/ffi/ext/libqalculate")
        .compile("libqalculate_ffi");

    println!("cargo:rustc-link-lib=qalculate");
    println!("cargo:rerun-if-changed=src/ffi/libqalculate.rs");
    println!("cargo:rerun-if-changed=src/ffi/ext/libqalculate/main.h");
    println!("cargo:rerun-if-changed=src/ffi/ext/libqalculate/main.cc");

    // Build FFI bindings for astal-wireplumber
    cxx_build::bridge("src/ffi/astalwp.rs")
        .file("src/ffi/ext/astalwp/main.cc")
        .file("src/ffi/ext/astalwp/data.cc")
        .file("src/ffi/ext/astalwp/event.cc")
        .include("src/ffi/ext/astalwp")
        .include("/usr/include/astal")
        .include("/usr/include/wireplumber-0.5")
        .include("/usr/include/libmount")
        .include("/usr/include/blkid")
        .include("/usr/include/glib-2.0")
        .include("/usr/lib/glib-2.0/include")
        .include("/usr/include/sysprof-6")
        .include("/usr/include/pipewire-0.3")
        .include("/usr/include/spa-0.2")
        .compile("astalwp_ffi");

    println!("cargo:rustc-link-lib=stdc++");
    println!("cargo:rustc-link-lib=astal-wireplumber");
    println!("cargo:rustc-link-lib=gobject-2.0");
    println!("cargo:rustc-link-lib=glib-2.0");
    println!("cargo:rerun-if-changed=src/ffi/astalwp.rs");
    println!("cargo:rerun-if-changed=src/ffi/ext/astalwp/data.h");
    println!("cargo:rerun-if-changed=src/ffi/ext/astalwp/data.cc");
    println!("cargo:rerun-if-changed=src/ffi/ext/astalwp/event.h");
    println!("cargo:rerun-if-changed=src/ffi/ext/astalwp/event.cc");
    println!("cargo:rerun-if-changed=src/ffi/ext/astalwp/main.h");
    println!("cargo:rerun-if-changed=src/ffi/ext/astalwp/main.cc");
}