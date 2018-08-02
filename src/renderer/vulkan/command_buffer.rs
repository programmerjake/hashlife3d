use super::{
    api, null_or_zero, DeviceWrapper, Result, VulkanDeviceIndexBuffer, VulkanDeviceVertexBuffer,
    VulkanError, VulkanStagingIndexBuffer, VulkanStagingVertexBuffer,
};
use renderer::{CommandBuffer, LoaderCommandBufferBuilder, RenderCommandBufferBuilder};
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
    pub unsafe fn new(
        device: &Arc<DeviceWrapper>,
        queue_family_index: u32,
        command_buffer_level: api::VkCommandBufferLevel,
    ) -> Result<Self> {
        let mut command_pool = null_or_zero();
        let command_pool = match device.vkCreateCommandPool.unwrap()(
            device.device,
            &api::VkCommandPoolCreateInfo {
                sType: api::VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
                pNext: null(),
                flags: 0,
                queueFamilyIndex: queue_family_index,
            },
            null(),
            &mut command_pool,
        ) {
            api::VK_SUCCESS => CommandPoolWrapper {
                device: device.clone(),
                command_pool: command_pool,
            },
            result => return Err(VulkanError::VulkanError(result)),
        };
        let mut command_buffer = null_mut();
        match device.vkAllocateCommandBuffers.unwrap()(
            device.device,
            &api::VkCommandBufferAllocateInfo {
                sType: api::VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
                pNext: null(),
                commandPool: command_pool.command_pool,
                level: command_buffer_level,
                commandBufferCount: 1,
            },
            &mut command_buffer,
        ) {
            api::VK_SUCCESS => Ok(CommandBufferWrapper {
                command_pool: command_pool,
                command_buffer: command_buffer,
            }),
            result => Err(VulkanError::VulkanError(result)),
        }
    }
    pub unsafe fn begin(
        self,
        flags: api::VkCommandBufferUsageFlags,
        inheritence_info: Option<&api::VkCommandBufferInheritanceInfo>,
    ) -> Result<CommandBufferWrapper> {
        match self.command_pool.device.vkBeginCommandBuffer.unwrap()(
            self.command_buffer,
            &api::VkCommandBufferBeginInfo {
                sType: api::VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
                pNext: null(),
                flags: flags,
                pInheritanceInfo: match inheritence_info {
                    Some(v) => v,
                    None => null(),
                },
            },
        ) {
            api::VK_SUCCESS => Ok(self),
            result => Err(VulkanError::VulkanError(result)),
        }
    }
    pub unsafe fn finish(self) -> Result<CommandBufferWrapper> {
        match self.command_pool.device.vkEndCommandBuffer.unwrap()(self.command_buffer) {
            api::VK_SUCCESS => Ok(self),
            result => Err(VulkanError::VulkanError(result)),
        }
    }
}

pub struct VulkanLoaderCommandBuffer {
    command_buffer: CommandBufferWrapper,
}

impl CommandBuffer for VulkanLoaderCommandBuffer {}

pub struct VulkanLoaderCommandBufferBuilder(VulkanLoaderCommandBuffer);

impl VulkanLoaderCommandBufferBuilder {
    pub unsafe fn new(device: &Arc<DeviceWrapper>, queue_family_index: u32) -> Result<Self> {
        Ok(Self {
            0: VulkanLoaderCommandBuffer {
                command_buffer: CommandBufferWrapper::new(
                    device,
                    queue_family_index,
                    api::VK_COMMAND_BUFFER_LEVEL_PRIMARY,
                )?.begin(api::VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT, None)?,
            },
        })
    }
}

impl LoaderCommandBufferBuilder for VulkanLoaderCommandBufferBuilder {
    type Error = VulkanError;
    type CommandBuffer = VulkanLoaderCommandBuffer;
    type StagingVertexBuffer = VulkanStagingVertexBuffer;
    type DeviceVertexBuffer = VulkanDeviceVertexBuffer;
    type StagingIndexBuffer = VulkanStagingIndexBuffer;
    type DeviceIndexBuffer = VulkanDeviceIndexBuffer;
    fn copy_vertex_buffer_to_device(
        staging_vertex_buffer: VulkanStagingVertexBuffer,
    ) -> Result<VulkanDeviceVertexBuffer> {
        unimplemented!()
    }
    fn copy_index_buffer_to_device(
        staging_index_buffer: VulkanStagingIndexBuffer,
    ) -> Result<VulkanDeviceIndexBuffer> {
        unimplemented!()
    }
    fn finish(self) -> Result<VulkanLoaderCommandBuffer> {
        let mut retval = self.0;
        retval.command_buffer = unsafe { retval.command_buffer.finish() }?;
        Ok(retval)
    }
}

struct VulkanRenderCommandBufferState {
    command_buffer: CommandBufferWrapper,
}

#[derive(Clone)]
pub struct VulkanRenderCommandBuffer(Arc<VulkanRenderCommandBufferState>);

unsafe impl Send for VulkanRenderCommandBuffer {}

impl CommandBuffer for VulkanRenderCommandBuffer {}

pub struct VulkanRenderCommandBufferBuilder(VulkanRenderCommandBufferState);

impl VulkanRenderCommandBufferBuilder {
    #[allow(unreachable_code)]
    pub unsafe fn new(device: &Arc<DeviceWrapper>, queue_family_index: u32) -> Result<Self> {
        let retval = Self {
            0: VulkanRenderCommandBufferState {
                command_buffer: CommandBufferWrapper::new(
                    device,
                    queue_family_index,
                    api::VK_COMMAND_BUFFER_LEVEL_SECONDARY,
                )?.begin(unimplemented!(), unimplemented!())?,
            },
        };
        unimplemented!()
    }
}

impl RenderCommandBufferBuilder for VulkanRenderCommandBufferBuilder {
    type Error = VulkanError;
    type CommandBuffer = VulkanRenderCommandBuffer;
    type DeviceVertexBuffer = VulkanDeviceVertexBuffer;
    type DeviceIndexBuffer = VulkanDeviceIndexBuffer;
    fn finish(self) -> Result<VulkanRenderCommandBuffer> {
        let mut retval = self.0;
        retval.command_buffer = unsafe { retval.command_buffer.finish() }?;
        unimplemented!()
    }
}
