// This file is part of Hashlife3d.
//
// Hashlife3d is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Hashlife3d is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with Hashlife3d.  If not, see <https://www.gnu.org/licenses/>
extern crate bindgen;
extern crate pkg_config;

use std::env;
use std::path::*;

fn get_out_path() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").unwrap())
}

fn main() -> Result<(), String> {
    let target = env::var("TARGET").unwrap();
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
    let out_path = get_out_path();
    println!("cargo:rerun-if-changed=sdl-wrapper.h");
    let mut bindings = bindgen::Builder::default()
        .header("sdl-wrapper.h")
        .whitelisted_function("SDL_.*")
        .whitelisted_type("SDL_.*")
        .whitelisted_var("SDL_.*")
        .whitelisted_var("KMOD_.*")
        .whitelisted_var("SDLK_.*")
        .opaque_type("FILE")
        .clang_arg("-target")
        .clang_arg(target.clone())
        .prepend_enum_name(false)
        .constified_enum(".*")
        .unstable_rust(true)
        .generate_comments(false);

    for path in &include_paths {
        bindings = bindings.clang_arg(format!("-I{}", path));
    }

    bindings
        .generate()
        .expect("Unable to generate sdl bindings")
        .write_to_file(out_path.join("sdl-bindings.rs"))
        .expect("Couldn't write sdl bindings!");
    Ok(())
}
