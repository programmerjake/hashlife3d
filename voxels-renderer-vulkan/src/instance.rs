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
use super::{api, InstanceFunctions, Result, VulkanError};
use std::os::raw::*;
use std::ptr::{null, null_mut};

#[allow(non_snake_case)]
pub struct InstanceWrapper {
    pub instance: api::VkInstance,
    pub instance_functions: InstanceFunctions,
    pub vkDestroyInstance: api::PFN_vkDestroyInstance,
    pub vkCreateDevice: api::PFN_vkCreateDevice,
    pub vkGetDeviceProcAddr: api::PFN_vkGetDeviceProcAddr,
    pub vkDestroySurfaceKHR: api::PFN_vkDestroySurfaceKHR,
    pub vkEnumeratePhysicalDevices: api::PFN_vkEnumeratePhysicalDevices,
    pub vkGetPhysicalDeviceSurfaceSupportKHR: api::PFN_vkGetPhysicalDeviceSurfaceSupportKHR,
    pub vkGetPhysicalDeviceQueueFamilyProperties: api::PFN_vkGetPhysicalDeviceQueueFamilyProperties,
    pub vkEnumerateDeviceExtensionProperties: api::PFN_vkEnumerateDeviceExtensionProperties,
    pub vkGetPhysicalDeviceSurfaceCapabilitiesKHR:
        api::PFN_vkGetPhysicalDeviceSurfaceCapabilitiesKHR,
    pub vkGetPhysicalDeviceSurfaceFormatsKHR: api::PFN_vkGetPhysicalDeviceSurfaceFormatsKHR,
    pub vkGetPhysicalDeviceFormatProperties: api::PFN_vkGetPhysicalDeviceFormatProperties,
    pub vkGetPhysicalDeviceSurfacePresentModesKHR:
        api::PFN_vkGetPhysicalDeviceSurfacePresentModesKHR,
    pub vkGetPhysicalDeviceMemoryProperties: api::PFN_vkGetPhysicalDeviceMemoryProperties,
    pub vkGetPhysicalDeviceImageFormatProperties: api::PFN_vkGetPhysicalDeviceImageFormatProperties,
}

unsafe impl Send for InstanceWrapper {}

unsafe impl Sync for InstanceWrapper {}

impl InstanceWrapper {
    pub unsafe fn new(
        instance_functions: InstanceFunctions,
        application_info: *const api::VkApplicationInfo,
        enabled_layer_names: &[*const c_char],
        enabled_extension_names: &[*const c_char],
    ) -> Result<Self> {
        let mut instance = null_mut();
        match instance_functions.vkCreateInstance.unwrap()(
            &api::VkInstanceCreateInfo {
                sType: api::VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
                pNext: null(),
                flags: 0,
                pApplicationInfo: application_info,
                enabledLayerCount: enabled_layer_names.len() as u32,
                ppEnabledLayerNames: enabled_layer_names.as_ptr(),
                enabledExtensionCount: enabled_extension_names.len() as u32,
                ppEnabledExtensionNames: enabled_extension_names.as_ptr(),
            },
            null(),
            &mut instance,
        ) {
            api::VK_SUCCESS => {
                let vk_get_instance_proc_addr = instance_functions.vkGetInstanceProcAddr;
                Ok(Self {
                    instance: instance,
                    instance_functions: instance_functions,
                    vkDestroyInstance: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkDestroyInstance,
                        PFN_vkDestroyInstance
                    ),
                    vkCreateDevice: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkCreateDevice,
                        PFN_vkCreateDevice
                    ),
                    vkGetDeviceProcAddr: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkGetDeviceProcAddr,
                        PFN_vkGetDeviceProcAddr
                    ),
                    vkDestroySurfaceKHR: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkDestroySurfaceKHR,
                        PFN_vkDestroySurfaceKHR
                    ),
                    vkEnumeratePhysicalDevices: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkEnumeratePhysicalDevices,
                        PFN_vkEnumeratePhysicalDevices
                    ),
                    vkGetPhysicalDeviceSurfaceSupportKHR: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkGetPhysicalDeviceSurfaceSupportKHR,
                        PFN_vkGetPhysicalDeviceSurfaceSupportKHR
                    ),
                    vkGetPhysicalDeviceQueueFamilyProperties: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkGetPhysicalDeviceQueueFamilyProperties,
                        PFN_vkGetPhysicalDeviceQueueFamilyProperties
                    ),
                    vkEnumerateDeviceExtensionProperties: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkEnumerateDeviceExtensionProperties,
                        PFN_vkEnumerateDeviceExtensionProperties
                    ),
                    vkGetPhysicalDeviceSurfaceCapabilitiesKHR: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkGetPhysicalDeviceSurfaceCapabilitiesKHR,
                        PFN_vkGetPhysicalDeviceSurfaceCapabilitiesKHR
                    ),
                    vkGetPhysicalDeviceSurfaceFormatsKHR: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkGetPhysicalDeviceSurfaceFormatsKHR,
                        PFN_vkGetPhysicalDeviceSurfaceFormatsKHR
                    ),
                    vkGetPhysicalDeviceFormatProperties: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkGetPhysicalDeviceFormatProperties,
                        PFN_vkGetPhysicalDeviceFormatProperties
                    ),
                    vkGetPhysicalDeviceSurfacePresentModesKHR: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkGetPhysicalDeviceSurfacePresentModesKHR,
                        PFN_vkGetPhysicalDeviceSurfacePresentModesKHR
                    ),
                    vkGetPhysicalDeviceMemoryProperties: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkGetPhysicalDeviceMemoryProperties,
                        PFN_vkGetPhysicalDeviceMemoryProperties
                    ),
                    vkGetPhysicalDeviceImageFormatProperties: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkGetPhysicalDeviceImageFormatProperties,
                        PFN_vkGetPhysicalDeviceImageFormatProperties
                    ),
                })
            }
            result => Err(VulkanError::VulkanError(result)),
        }
    }
}

impl Drop for InstanceWrapper {
    fn drop(&mut self) {
        unsafe {
            self.vkDestroyInstance.unwrap()(self.instance, null());
        }
    }
}
