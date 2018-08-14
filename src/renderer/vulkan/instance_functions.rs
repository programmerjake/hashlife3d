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
use super::api;
use std::ffi::CStr;
use std::mem;
use std::os::raw::*;
use std::ptr::*;

pub unsafe fn get_instance_fn(
    vk_get_instance_proc_addr: api::PFN_vkGetInstanceProcAddr,
    instance: api::VkInstance,
    name: &[u8],
) -> api::PFN_vkVoidFunction {
    let name = CStr::from_bytes_with_nul(name).unwrap();
    match vk_get_instance_proc_addr.unwrap()(instance, name.as_ptr()) {
        Some(retval) => Some(retval),
        None => panic!(
            "vkGetInstanceProcAddr failed: function not found: {}",
            name.to_string_lossy()
        ),
    }
}

macro_rules! get_instance_fn {
    ($vk_get_instance_proc_addr:expr, $instance:expr, $name:ident) => {{
        use renderer::vulkan::api::*;
        ::std::mem::transmute::<api::PFN_vkVoidFunction, concat_idents!(PFN_, $name)>(
            ::renderer::vulkan::instance_functions::get_instance_fn(
                $vk_get_instance_proc_addr,
                $instance,
                concat!(stringify!($name), "\0").as_bytes(),
            ),
        )
    }};
}

#[allow(non_snake_case)]
#[derive(Copy, Clone)]
pub struct InstanceFunctions {
    pub vkGetInstanceProcAddr: api::PFN_vkGetInstanceProcAddr,
    pub vkCreateInstance: api::PFN_vkCreateInstance,
}

unsafe impl Sync for InstanceFunctions {}
unsafe impl Send for InstanceFunctions {}

impl InstanceFunctions {
    pub unsafe fn new(vk_get_instance_proc_addr: *const c_void) -> Self {
        let vk_get_instance_proc_addr: api::PFN_vkGetInstanceProcAddr =
            mem::transmute(vk_get_instance_proc_addr);
        assert!(vk_get_instance_proc_addr.is_some());
        Self {
            vkGetInstanceProcAddr: vk_get_instance_proc_addr,
            vkCreateInstance: get_instance_fn!(
                vk_get_instance_proc_addr,
                null_mut(),
                vkCreateInstance
            ),
        }
    }
}
