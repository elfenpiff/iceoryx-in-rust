extern crate bindgen;
extern crate cc;

use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to look for shared libraries in the specified directory
    //println!("cargo:rustc-link-search=/path/to/lib");

    // Tell cargo to tell rustc to link the system bzip2
    // shared library.
    #[cfg(any(target_os = "linux"))]
    println!("cargo:rustc-link-lib=acl");
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    println!("cargo:rustc-link-lib=pthread");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=src/posix.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("src/c/posix.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("posix_generated.rs"))
        .expect("Couldn't write bindings!");

    cc::Build::new()
        .file("src/c/scandir.c")
        .compile("libscandir.a");

    cc::Build::new()
        .file("src/c/socket_macros.c")
        .compile("libsocket_macros.a");
}
