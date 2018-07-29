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
                        vkDestroyInstance
                    ),
                    vkCreateDevice: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkCreateDevice
                    ),
                    vkGetDeviceProcAddr: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkGetDeviceProcAddr
                    ),
                    vkDestroySurfaceKHR: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkDestroySurfaceKHR
                    ),
                    vkEnumeratePhysicalDevices: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkEnumeratePhysicalDevices
                    ),
                    vkGetPhysicalDeviceSurfaceSupportKHR: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkGetPhysicalDeviceSurfaceSupportKHR
                    ),
                    vkGetPhysicalDeviceQueueFamilyProperties: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkGetPhysicalDeviceQueueFamilyProperties
                    ),
                    vkEnumerateDeviceExtensionProperties: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkEnumerateDeviceExtensionProperties
                    ),
                    vkGetPhysicalDeviceSurfaceCapabilitiesKHR: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkGetPhysicalDeviceSurfaceCapabilitiesKHR
                    ),
                    vkGetPhysicalDeviceSurfaceFormatsKHR: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkGetPhysicalDeviceSurfaceFormatsKHR
                    ),
                    vkGetPhysicalDeviceFormatProperties: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkGetPhysicalDeviceFormatProperties
                    ),
                    vkGetPhysicalDeviceSurfacePresentModesKHR: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkGetPhysicalDeviceSurfacePresentModesKHR
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
