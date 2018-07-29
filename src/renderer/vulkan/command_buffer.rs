use super::{api, null_or_zero, DeviceWrapper, Result, VulkanError};
use renderer::{CommandBuffer, CommandBufferBuilder};
use std::ptr::{null, null_mut};
use std::sync::Arc;

pub struct CommandPoolWrapper {
    pub device: Arc<DeviceWrapper>,
    pub command_pool: api::VkCommandPool,
}

unsafe impl Send for CommandPoolWrapper {}

impl Drop for CommandPoolWrapper {
    fn drop(&mut self) {
        unsafe {
            self.device.vkDestroyCommandPool.unwrap()(
                self.device.device,
                self.command_pool,
                null(),
            );
        }
    }
}

pub struct CommandBufferWrapper {
    pub command_pool: CommandPoolWrapper,
    pub command_buffer: api::VkCommandBuffer,
}

impl Drop for CommandBufferWrapper {
    fn drop(&mut self) {
        unsafe {
            self.command_pool.device.vkFreeCommandBuffers.unwrap()(
                self.command_pool.device.device,
                self.command_pool.command_pool,
                1,
                &self.command_buffer,
            );
        }
    }
}

unsafe impl Send for CommandBufferWrapper {}

impl CommandBufferWrapper {
    pub fn new(
        device: &Arc<DeviceWrapper>,
        queue_family_index: u32,
        command_buffer_level: api::VkCommandBufferLevel,
    ) -> Result<Self> {
        let mut command_pool = null_or_zero();
        let command_pool = match unsafe {
            device.vkCreateCommandPool.unwrap()(
                device.device,
                &api::VkCommandPoolCreateInfo {
                    sType: api::VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
                    pNext: null(),
                    flags: 0,
                    queueFamilyIndex: queue_family_index,
                },
                null(),
                &mut command_pool,
            )
        } {
            api::VK_SUCCESS => CommandPoolWrapper {
                device: device.clone(),
                command_pool: command_pool,
            },
            result => return Err(VulkanError::VulkanError(result)),
        };
        let mut command_buffer = null_mut();
        match unsafe {
            device.vkAllocateCommandBuffers.unwrap()(
                device.device,
                &api::VkCommandBufferAllocateInfo {
                    sType: api::VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
                    pNext: null(),
                    commandPool: command_pool.command_pool,
                    level: command_buffer_level,
                    commandBufferCount: 1,
                },
                &mut command_buffer,
            )
        } {
            api::VK_SUCCESS => Ok(CommandBufferWrapper {
                command_pool: command_pool,
                command_buffer: command_buffer,
            }),
            result => Err(VulkanError::VulkanError(result)),
        }
    }
}

pub struct VulkanCommandBuffer {
    command_buffer: CommandBufferWrapper,
}

impl CommandBuffer for VulkanCommandBuffer {}

pub struct VulkanCommandBufferBuilder(VulkanCommandBuffer);

impl CommandBufferBuilder for VulkanCommandBufferBuilder {
    type Error = VulkanError;
    type CommandBuffer = VulkanCommandBuffer;
    fn finish(self) -> Result<VulkanCommandBuffer> {
        match unsafe {
            self.0
                .command_buffer
                .command_pool
                .device
                .vkEndCommandBuffer
                .unwrap()(self.0.command_buffer.command_buffer)
        } {
            api::VK_SUCCESS => Ok(self.0),
            result => Err(VulkanError::VulkanError(result)),
        }
    }
}
