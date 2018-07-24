extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("cargo:rustc-link-lib=SDL2");
    println!("cargo:rustc-link-lib=SDL2main");
    println!("cargo:rerun-if-changed=sdl-wrapper.h");
    bindgen::Builder::default()
        .header("sdl-wrapper.h")
        .clang_arg("-I/usr/include/SDL2")
        .clang_arg("-D_REENTRANT")
        .whitelist_function("SDL_.*")
        .whitelist_type("SDL_.*")
        .whitelist_var("SDL_.*")
        .whitelist_var("KMOD_.*")
        .whitelist_var("SDLK_.*")
        .blacklist_type("VkSurfaceKHR.*")
        .raw_line("pub type VkSurfaceKHR = u64;")
        .opaque_type("FILE")
        .rustfmt_bindings(true)
        .prepend_enum_name(false)
        .generate()
        .expect("Unable to generate sdl bindings")
        .write_to_file(out_path.join("sdl-bindings.rs"))
        .expect("Couldn't write sdl bindings!");

    println!("cargo:rerun-if-changed=vulkan-wrapper.h");
    bindgen::Builder::default()
        .header("vulkan-wrapper.h")
        .whitelist_type("Vk.*")
        .whitelist_type("PFN_vk.*")
        .whitelist_var("VK_.*")
        .rustfmt_bindings(true)
        .prepend_enum_name(false)
        .generate()
        .expect("Unable to generate vulkan bindings")
        .write_to_file(out_path.join("vulkan-bindings.rs"))
        .expect("Couldn't write vulkan bindings!");
        
    println!("cargo:rerun-if-changed=gles2-wrapper.h");
    bindgen::Builder::default()
        .header("gles2-wrapper.h")
        .whitelist_type("PFNGL.*")
        .whitelist_var("GL_.*")
        .blacklist_type("khronos_.*")
        .rustfmt_bindings(true)
        .prepend_enum_name(false)
        .generate()
        .expect("Unable to generate gles2 bindings")
        .write_to_file(out_path.join("gles2-bindings.rs"))
        .expect("Couldn't write gles2 bindings!");
}
