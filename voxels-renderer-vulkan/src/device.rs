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
use super::{api, api::*, InstanceWrapper, Result, VulkanError};
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
        Some(retval) => Some(mem::transmute(retval)),
        None => panic!(
            "vkGetDeviceProcAddr failed: function not found: {}",
            name.to_string_lossy()
        ),
    }
}

macro_rules! get_device_fn {
    ($vk_get_device_proc_addr:expr, $device:expr, $name:ident, $pfn_name:ident) => {{
        use self::api::*;
        assert_eq!(concat!("PFN_", stringify!($name)), stringify!($pfn_name));
        mem::transmute::<api::PFN_vkVoidFunction, $pfn_name>(self::get_device_fn(
            $vk_get_device_proc_addr,
            $device,
            concat!(stringify!($name), "\0").as_bytes(),
        ))
    }};
}

macro_rules! make_device_wrapper {
    ($(($names:ident, $pfn_names:ident),)*) => {
        #[allow(non_snake_case)]
        pub struct DeviceWrapper {
            pub device: api::VkDevice,
            pub instance: Arc<InstanceWrapper>,
            $(pub $names: $pfn_names,)*
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
                            $($names: get_device_fn!(vk_get_device_proc_addr, device, $names, $pfn_names),)*
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
    };
}

make_device_wrapper!(
    (vkAcquireNextImageKHR, PFN_vkAcquireNextImageKHR),
    (vkAllocateCommandBuffers, PFN_vkAllocateCommandBuffers),
    (vkAllocateDescriptorSets, PFN_vkAllocateDescriptorSets),
    (vkAllocateMemory, PFN_vkAllocateMemory),
    (vkBeginCommandBuffer, PFN_vkBeginCommandBuffer),
    (vkBindBufferMemory, PFN_vkBindBufferMemory),
    (vkBindImageMemory, PFN_vkBindImageMemory),
    (vkCmdBeginRenderPass, PFN_vkCmdBeginRenderPass),
    (vkCmdBindDescriptorSets, PFN_vkCmdBindDescriptorSets),
    (vkCmdBindIndexBuffer, PFN_vkCmdBindIndexBuffer),
    (vkCmdBindPipeline, PFN_vkCmdBindPipeline),
    (vkCmdBindVertexBuffers, PFN_vkCmdBindVertexBuffers),
    (vkCmdClearColorImage, PFN_vkCmdClearColorImage),
    (vkCmdCopyBuffer, PFN_vkCmdCopyBuffer),
    (vkCmdCopyBufferToImage, PFN_vkCmdCopyBufferToImage),
    (vkCmdDrawIndexed, PFN_vkCmdDrawIndexed),
    (vkCmdEndRenderPass, PFN_vkCmdEndRenderPass),
    (vkCmdExecuteCommands, PFN_vkCmdExecuteCommands),
    (vkCmdPipelineBarrier, PFN_vkCmdPipelineBarrier),
    (vkCmdPushConstants, PFN_vkCmdPushConstants),
    (vkCmdSetScissor, PFN_vkCmdSetScissor),
    (vkCmdSetViewport, PFN_vkCmdSetViewport),
    (vkCreateBuffer, PFN_vkCreateBuffer),
    (vkCreateCommandPool, PFN_vkCreateCommandPool),
    (vkCreateDescriptorPool, PFN_vkCreateDescriptorPool),
    (vkCreateDescriptorSetLayout, PFN_vkCreateDescriptorSetLayout),
    (vkCreateFence, PFN_vkCreateFence),
    (vkCreateFramebuffer, PFN_vkCreateFramebuffer),
    (vkCreateGraphicsPipelines, PFN_vkCreateGraphicsPipelines),
    (vkCreateImage, PFN_vkCreateImage),
    (vkCreateImageView, PFN_vkCreateImageView),
    (vkCreatePipelineLayout, PFN_vkCreatePipelineLayout),
    (vkCreateRenderPass, PFN_vkCreateRenderPass),
    (vkCreateSampler, PFN_vkCreateSampler),
    (vkCreateSemaphore, PFN_vkCreateSemaphore),
    (vkCreateShaderModule, PFN_vkCreateShaderModule),
    (vkCreateSwapchainKHR, PFN_vkCreateSwapchainKHR),
    (vkDestroyBuffer, PFN_vkDestroyBuffer),
    (vkDestroyCommandPool, PFN_vkDestroyCommandPool),
    (vkDestroyDescriptorPool, PFN_vkDestroyDescriptorPool),
    (
        vkDestroyDescriptorSetLayout,
        PFN_vkDestroyDescriptorSetLayout
    ),
    (vkDestroyDevice, PFN_vkDestroyDevice),
    (vkDestroyFence, PFN_vkDestroyFence),
    (vkDestroyFramebuffer, PFN_vkDestroyFramebuffer),
    (vkDestroyImage, PFN_vkDestroyImage),
    (vkDestroyImageView, PFN_vkDestroyImageView),
    (vkDestroyPipeline, PFN_vkDestroyPipeline),
    (vkDestroyPipelineLayout, PFN_vkDestroyPipelineLayout),
    (vkDestroyRenderPass, PFN_vkDestroyRenderPass),
    (vkDestroySampler, PFN_vkDestroySampler),
    (vkDestroySemaphore, PFN_vkDestroySemaphore),
    (vkDestroyShaderModule, PFN_vkDestroyShaderModule),
    (vkDestroySwapchainKHR, PFN_vkDestroySwapchainKHR),
    (vkDeviceWaitIdle, PFN_vkDeviceWaitIdle),
    (vkEndCommandBuffer, PFN_vkEndCommandBuffer),
    (vkFreeCommandBuffers, PFN_vkFreeCommandBuffers),
    (vkFreeMemory, PFN_vkFreeMemory),
    (
        vkGetBufferMemoryRequirements,
        PFN_vkGetBufferMemoryRequirements
    ),
    (vkGetDeviceQueue, PFN_vkGetDeviceQueue),
    (vkGetFenceStatus, PFN_vkGetFenceStatus),
    (
        vkGetImageMemoryRequirements,
        PFN_vkGetImageMemoryRequirements
    ),
    (vkGetSwapchainImagesKHR, PFN_vkGetSwapchainImagesKHR),
    (vkMapMemory, PFN_vkMapMemory),
    (vkQueuePresentKHR, PFN_vkQueuePresentKHR),
    (vkQueueSubmit, PFN_vkQueueSubmit),
    (vkUnmapMemory, PFN_vkUnmapMemory),
    (vkUpdateDescriptorSets, PFN_vkUpdateDescriptorSets),
    (vkWaitForFences, PFN_vkWaitForFences),
);
