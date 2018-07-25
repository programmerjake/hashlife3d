extern crate bindgen;
extern crate pkg_config;

use std::env;
use std::error;
use std::path::PathBuf;

fn main() -> Result<(), Box<error::Error>> {
    let target = env::var("TARGET").unwrap();
    let host = env::var("HOST").unwrap();
    let mut include_paths = Vec::new();
    if let Ok(include_path) = env::var("SDL2_INCLUDE_PATH") {
        include_paths.push(include_path);
    }
    for path in pkg_config::Config::new()
        .print_system_libs(false)
        .probe("sdl2")
        .map_err(|e| format!("{}", e))?
        .include_paths
    {
        include_paths.push(path.to_str().unwrap().to_string());
    }
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("cargo:rerun-if-changed=sdl-wrapper.h");
    let mut sdl_bindings = Some(
        bindgen::Builder::default()
            .header("sdl-wrapper.h")
            .whitelisted_function("SDL_.*")
            .whitelisted_type("SDL_.*")
            .whitelisted_var("SDL_.*")
            .whitelisted_var("KMOD_.*")
            .whitelisted_var("SDLK_.*")
            .opaque_type("FILE"),
    );

    println!("cargo:rerun-if-changed=vulkan-wrapper.h");
    let mut vulkan_bindings = Some(
        bindgen::Builder::default()
            .header("vulkan-wrapper.h")
            .whitelisted_type("Vk.*")
            .whitelisted_type("PFN_vk.*")
            .whitelisted_var("VK_.*"),
    );

    println!("cargo:rerun-if-changed=gles2-wrapper.h");
    let mut gles2_bindings = Some(
        bindgen::Builder::default()
            .header("gles2-wrapper.h")
            .whitelisted_type("PFNGL.*")
            .whitelisted_var("GL_.*"),
    );

    for bindings in &mut [&mut sdl_bindings, &mut vulkan_bindings, &mut gles2_bindings] {
        **bindings = Some(
            bindings
                .take()
                .unwrap()
                .clang_arg("-target")
                .clang_arg(target.clone())
                .prepend_enum_name(false)
                .constified_enum(".*")
                .unstable_rust(true),
        );
        for path in &include_paths {
            **bindings = Some(bindings.take().unwrap().clang_arg(format!("-I{}", path)));
        }
    }

    sdl_bindings
        .unwrap()
        .generate()
        .expect("Unable to generate sdl bindings")
        .write_to_file(out_path.join("sdl-bindings.rs"))
        .expect("Couldn't write sdl bindings!");
    vulkan_bindings
        .unwrap()
        .generate()
        .expect("Unable to generate vulkan bindings")
        .write_to_file(out_path.join("vulkan-bindings.rs"))
        .expect("Couldn't write vulkan bindings!");
    gles2_bindings
        .unwrap()
        .generate()
        .expect("Unable to generate gles2 bindings")
        .write_to_file(out_path.join("gles2-bindings.rs"))
        .expect("Couldn't write gles2 bindings!");
    Ok(())
}
