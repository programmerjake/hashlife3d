extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=SDL2");
    println!("cargo:rustc-link-lib=SDL2main");
    println!("cargo:rerun-if-changed=wrapper.h");
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-I/usr/include/SDL2")
        .clang_arg("-D_REENTRANT")
        .whitelist_function("SDL_.*")
        .whitelist_type("SDL_.*")
        .whitelist_var("SDL_.*")
        .blacklist_type("VkSurfaceKHR.*")
        .raw_line("pub type VkSurfaceKHR = u64;")
        .opaque_type("FILE")
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
