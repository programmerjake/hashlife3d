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

use std::env;
use std::path::*;

fn get_out_path() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").unwrap())
}

fn main() -> Result<(), String> {
    let target = env::var("TARGET").unwrap();
    let out_path = get_out_path();
    println!("cargo:rerun-if-changed=gles2-wrapper.h");
    bindgen::Builder::default()
        .header("gles2-wrapper.h")
        .whitelisted_type("PFNGL.*")
        .whitelisted_var("GL_.*")
        .clang_arg("-target")
        .clang_arg(target)
        .prepend_enum_name(false)
        .constified_enum(".*")
        .unstable_rust(true)
        .generate()
        .expect("Unable to generate gles2 bindings")
        .write_to_file(out_path.join("gles2-bindings.rs"))
        .expect("Couldn't write gles2 bindings!");
    Ok(())
}
