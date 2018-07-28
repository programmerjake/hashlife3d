use super::{api, InstanceWrapper, Result, VulkanError};
use std::ffi::CStr;
use std::mem;
use std::os::raw::*;
use std::ptr::{null, null_mut};
use std::sync::Arc;

unsafe fn get_device_fn(
    vk_get_device_proc_addr: api::PFN_vkGetDeviceProcAddr,
    device: api::VkDevice,
    name: &[u8],
) -> api::PFN_vkVoidFunction {
    let name = CStr::from_bytes_with_nul(name).unwrap();
    match vk_get_device_proc_addr.unwrap()(device, name.as_ptr()) {
        Some(retval) => Some(retval),
        None => panic!(
            "vkGetDeviceProcAddr failed: function not found: {}",
            name.to_string_lossy()
        ),
    }
}

macro_rules! get_device_fn {
    ($vk_get_device_proc_addr:expr, $device:expr, $name:ident) => {{
        use self::api::*;
        mem::transmute::<api::PFN_vkVoidFunction, concat_idents!(PFN_, $name)>(self::get_device_fn(
            $vk_get_device_proc_addr,
            $device,
            concat!(stringify!($name), "\0").as_bytes(),
        ))
    }};
}

#[allow(non_snake_case)]
pub struct DeviceWrapper {
    pub device: api::VkDevice,
    pub instance: Arc<InstanceWrapper>,
    pub vkDestroyDevice: api::PFN_vkDestroyDevice,
    pub vkDeviceWaitIdle: api::PFN_vkDeviceWaitIdle,
    pub vkWaitForFences: api::PFN_vkWaitForFences,
    pub vkCreateFence: api::PFN_vkCreateFence,
    pub vkDestroyFence: api::PFN_vkDestroyFence,
    pub vkGetDeviceQueue: api::PFN_vkGetDeviceQueue,
    pub vkCreateShaderModule: api::PFN_vkCreateShaderModule,
    pub vkDestroyShaderModule: api::PFN_vkDestroyShaderModule,
    pub vkDestroyPipeline: api::PFN_vkDestroyPipeline,
    pub vkCreateGraphicsPipelines: api::PFN_vkCreateGraphicsPipelines,
    pub vkDestroyDescriptorSetLayout: api::PFN_vkDestroyDescriptorSetLayout,
    pub vkCreateDescriptorSetLayout: api::PFN_vkCreateDescriptorSetLayout,
    pub vkDestroyPipelineLayout: api::PFN_vkDestroyPipelineLayout,
    pub vkCreatePipelineLayout: api::PFN_vkCreatePipelineLayout,
    pub vkDestroyRenderPass: api::PFN_vkDestroyRenderPass,
    pub vkCreateRenderPass: api::PFN_vkCreateRenderPass,
}

unsafe impl Sync for DeviceWrapper {}
unsafe impl Send for DeviceWrapper {}

impl DeviceWrapper {
    pub unsafe fn new(
        instance: Arc<InstanceWrapper>,
        physical_device: api::VkPhysicalDevice,
        queue_create_infos: &[api::VkDeviceQueueCreateInfo],
        enabled_extension_names: &[*const c_char],
        enabled_features: Option<&api::VkPhysicalDeviceFeatures>,
    ) -> Result<Self> {
        let mut device = null_mut();
        match instance.vkCreateDevice.unwrap()(
            physical_device,
            &api::VkDeviceCreateInfo {
                sType: api::VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
                pNext: null(),
                flags: 0,
                queueCreateInfoCount: queue_create_infos.len() as u32,
                pQueueCreateInfos: queue_create_infos.as_ptr(),
                enabledLayerCount: 0,
                ppEnabledLayerNames: null(),
                enabledExtensionCount: enabled_extension_names.len() as u32,
                ppEnabledExtensionNames: enabled_extension_names.as_ptr(),
                pEnabledFeatures: match enabled_features {
                    Some(v) => v,
                    None => null(),
                },
            },
            null(),
            &mut device,
        ) {
            api::VK_SUCCESS => {
                let vk_get_device_proc_addr = instance.vkGetDeviceProcAddr;
                Ok(Self {
                    device: device,
                    instance: instance,
                    vkDestroyDevice: get_device_fn!(
                        vk_get_device_proc_addr,
                        device,
                        vkDestroyDevice
                    ),
                    vkDeviceWaitIdle: get_device_fn!(
                        vk_get_device_proc_addr,
                        device,
                        vkDeviceWaitIdle
                    ),
                    vkWaitForFences: get_device_fn!(
                        vk_get_device_proc_addr,
                        device,
                        vkWaitForFences
                    ),
                    vkCreateFence: get_device_fn!(vk_get_device_proc_addr, device, vkCreateFence),
                    vkDestroyFence: get_device_fn!(vk_get_device_proc_addr, device, vkDestroyFence),
                    vkGetDeviceQueue: get_device_fn!(
                        vk_get_device_proc_addr,
                        device,
                        vkGetDeviceQueue
                    ),
                    vkCreateShaderModule: get_device_fn!(
                        vk_get_device_proc_addr,
                        device,
                        vkCreateShaderModule
                    ),
                    vkDestroyShaderModule: get_device_fn!(
                        vk_get_device_proc_addr,
                        device,
                        vkDestroyShaderModule
                    ),
                    vkDestroyPipeline: get_device_fn!(
                        vk_get_device_proc_addr,
                        device,
                        vkDestroyPipeline
                    ),
                    vkCreateGraphicsPipelines: get_device_fn!(
                        vk_get_device_proc_addr,
                        device,
                        vkCreateGraphicsPipelines
                    ),
                    vkDestroyDescriptorSetLayout: get_device_fn!(
                        vk_get_device_proc_addr,
                        device,
                        vkDestroyDescriptorSetLayout
                    ),
                    vkCreateDescriptorSetLayout: get_device_fn!(
                        vk_get_device_proc_addr,
                        device,
                        vkCreateDescriptorSetLayout
                    ),
                    vkDestroyPipelineLayout: get_device_fn!(
                        vk_get_device_proc_addr,
                        device,
                        vkDestroyPipelineLayout
                    ),
                    vkCreatePipelineLayout: get_device_fn!(
                        vk_get_device_proc_addr,
                        device,
                        vkCreatePipelineLayout
                    ),
                    vkDestroyRenderPass: get_device_fn!(
                        vk_get_device_proc_addr,
                        device,
                        vkDestroyRenderPass
                    ),
                    vkCreateRenderPass: get_device_fn!(
                        vk_get_device_proc_addr,
                        device,
                        vkCreateRenderPass
                    ),
                })
            }
            result => Err(VulkanError::VulkanError(result)),
        }
    }
}

impl Drop for DeviceWrapper {
    fn drop(&mut self) {
        unsafe {
            self.vkDeviceWaitIdle.unwrap()(self.device);
            self.vkDestroyDevice.unwrap()(self.device, null());
        }
    }
}
