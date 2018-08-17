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
extern crate glsl_to_spirv;

use std::env;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::mem;
use std::path::*;
use std::str;

fn get_out_path() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").unwrap())
}

fn main() -> Result<(), String> {
    let target = env::var("TARGET").unwrap();
    let out_path = get_out_path();
    struct VulkanShader {
        file_name: &'static str,
        stage: glsl_to_spirv::ShaderType,
    };
    let shaders_path = PathBuf::from("shaders");
    for VulkanShader { file_name, stage } in &[
        VulkanShader {
            file_name: "main.vert",
            stage: glsl_to_spirv::ShaderType::Vertex,
        },
        VulkanShader {
            file_name: "main.frag",
            stage: glsl_to_spirv::ShaderType::Fragment,
        },
    ] {
        let input_file = shaders_path.join(file_name);
        println!("cargo:rerun-if-changed={}", input_file.to_str().unwrap());
        let source = fs::read(input_file).map_err(|e| format!("{}", e))?;
        let mut output = match glsl_to_spirv::compile(
            str::from_utf8(&source).map_err(|e| format!("{}", e))?,
            stage.clone(),
        ) {
            Err(error) => {
                eprintln!("{}", error);
                return Err("shader compile failed".into());
            }
            Ok(v) => v,
        };
        output
            .seek(SeekFrom::Start(0))
            .map_err(|e| format!("{}", e))?;
        let mut buffer = Vec::new();
        output
            .read_to_end(&mut buffer)
            .map_err(|e| format!("{}", e))?;
        mem::drop(output);
        fs::write(out_path.join(String::from(*file_name) + ".spv"), buffer)
            .map_err(|e| format!("{}", e))?;
    }
    println!("cargo:rerun-if-changed=vulkan-wrapper.h");
    bindgen::Builder::default()
        .header("vulkan-wrapper.h")
        .whitelisted_type("Vk.*")
        .whitelisted_type("PFN_vk.*")
        .whitelisted_var("VK_.*")
        .clang_arg("-target")
        .clang_arg(target.clone())
        .prepend_enum_name(false)
        .constified_enum(".*")
        .unstable_rust(true)
        .generate()
        .expect("Unable to generate vulkan bindings")
        .write_to_file(out_path.join("vulkan-bindings.rs"))
        .expect("Couldn't write vulkan bindings!");
    Ok(())
}
