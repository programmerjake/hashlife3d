use super::{
    api, get_mut_vulkan_device_index_buffer_implementation,
    get_mut_vulkan_device_vertex_buffer_implementation,
    into_vulkan_staging_index_buffer_implementation,
    into_vulkan_staging_vertex_buffer_implementation, null_or_zero, BufferWrapper,
    DeviceMemoryPoolAllocation, DeviceWrapper, Result, VulkanDeviceIndexBuffer,
    VulkanDeviceVertexBuffer, VulkanError, VulkanStagingIndexBuffer,
    VulkanStagingIndexBufferImplementation, VulkanStagingVertexBuffer,
    VulkanStagingVertexBufferImplementation,
};
use renderer::{
    CommandBuffer, IndexBufferElement, LoaderCommandBufferBuilder, RenderCommandBufferBuilder,
    VertexBufferElement,
};
use std::mem;
use std::ptr::{null, null_mut};
use std::sync::atomic::*;
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
    pub queue_family_index: u32,
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
                queue_family_index: queue_family_index,
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

#[derive(Clone)]
pub struct CommandBufferSubmitTracker {
    submitted_flag: Arc<AtomicBool>,
}

impl CommandBufferSubmitTracker {
    pub fn new() -> Self {
        Self {
            submitted_flag: Arc::new(AtomicBool::new(false)),
        }
    }
    pub fn assert_submitted(&self) {
        assert!(self.submitted_flag.load(Ordering::Acquire));
    }
    pub unsafe fn set_submitted(&self) {
        self.submitted_flag.store(true, Ordering::Release);
    }
}

struct CommandBufferReferencedObjects {
    required_command_buffers: Vec<CommandBufferSubmitTracker>,
    device_memory_allocations: Vec<DeviceMemoryPoolAllocation>,
    shared_device_memory_allocations: Vec<Arc<DeviceMemoryPoolAllocation>>,
    buffers: Vec<BufferWrapper>,
    shared_buffers: Vec<Arc<BufferWrapper>>,
}

impl Default for CommandBufferReferencedObjects {
    fn default() -> Self {
        Self {
            required_command_buffers: Vec::new(),
            device_memory_allocations: Vec::new(),
            shared_device_memory_allocations: Vec::new(),
            buffers: Vec::new(),
            shared_buffers: Vec::new(),
        }
    }
}

pub struct VulkanLoaderCommandBuffer {
    command_buffer: CommandBufferWrapper,
    submit_tracker: CommandBufferSubmitTracker,
    referenced_objects: CommandBufferReferencedObjects,
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
                submit_tracker: CommandBufferSubmitTracker::new(),
                referenced_objects: Default::default(),
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
        &mut self,
        staging_vertex_buffer: VulkanStagingVertexBuffer,
    ) -> Result<VulkanDeviceVertexBuffer> {
        let command_buffer = &self.0.command_buffer;
        let device = &command_buffer.command_pool.device;
        let VulkanStagingVertexBufferImplementation {
            staging_buffer,
            staging_device_memory,
            mut device_buffer,
            element_count,
        } = into_vulkan_staging_vertex_buffer_implementation(staging_vertex_buffer);
        unsafe {
            let device_buffer_implementation =
                get_mut_vulkan_device_vertex_buffer_implementation(&mut device_buffer);
            device_buffer_implementation.submit_tracker = Some(self.0.submit_tracker.clone());
            device.vkCmdCopyBuffer.unwrap()(
                command_buffer.command_buffer,
                staging_buffer.buffer,
                device_buffer_implementation.buffer.buffer,
                1,
                &api::VkBufferCopy {
                    srcOffset: 0,
                    dstOffset: 0,
                    size: element_count as u64 * mem::size_of::<VertexBufferElement>() as u64,
                },
            );
            self.0.referenced_objects.buffers.push(staging_buffer);
            self.0
                .referenced_objects
                .device_memory_allocations
                .push(staging_device_memory);
            self.0
                .referenced_objects
                .shared_buffers
                .push(device_buffer_implementation.buffer.clone());
            self.0
                .referenced_objects
                .shared_device_memory_allocations
                .push(device_buffer_implementation.device_memory.clone());
            device.vkCmdPipelineBarrier.unwrap()(
                command_buffer.command_buffer,
                api::VK_PIPELINE_STAGE_TRANSFER_BIT,
                api::VK_PIPELINE_STAGE_VERTEX_INPUT_BIT,
                0,
                0,
                null(),
                1,
                &api::VkBufferMemoryBarrier {
                    sType: api::VK_STRUCTURE_TYPE_BUFFER_MEMORY_BARRIER,
                    pNext: null(),
                    srcAccessMask: api::VK_ACCESS_TRANSFER_WRITE_BIT,
                    dstAccessMask: api::VK_ACCESS_VERTEX_ATTRIBUTE_READ_BIT,
                    srcQueueFamilyIndex: self.0.command_buffer.queue_family_index,
                    dstQueueFamilyIndex: self.0.command_buffer.queue_family_index,
                    buffer: device_buffer_implementation.buffer.buffer,
                    offset: 0,
                    size: api::VK_WHOLE_SIZE as u64,
                },
                0,
                null(),
            );
        }
        Ok(device_buffer)
    }
    fn copy_index_buffer_to_device(
        &mut self,
        staging_index_buffer: VulkanStagingIndexBuffer,
    ) -> Result<VulkanDeviceIndexBuffer> {
        let command_buffer = &self.0.command_buffer;
        let device = &command_buffer.command_pool.device;
        let VulkanStagingIndexBufferImplementation {
            staging_buffer,
            staging_device_memory,
            mut device_buffer,
            element_count,
        } = into_vulkan_staging_index_buffer_implementation(staging_index_buffer);
        unsafe {
            let device_buffer_implementation =
                get_mut_vulkan_device_index_buffer_implementation(&mut device_buffer);
            device_buffer_implementation.submit_tracker = Some(self.0.submit_tracker.clone());
            device.vkCmdCopyBuffer.unwrap()(
                command_buffer.command_buffer,
                staging_buffer.buffer,
                device_buffer_implementation.buffer.buffer,
                1,
                &api::VkBufferCopy {
                    srcOffset: 0,
                    dstOffset: 0,
                    size: element_count as u64 * mem::size_of::<IndexBufferElement>() as u64,
                },
            );
            self.0.referenced_objects.buffers.push(staging_buffer);
            self.0
                .referenced_objects
                .device_memory_allocations
                .push(staging_device_memory);
            self.0
                .referenced_objects
                .shared_buffers
                .push(device_buffer_implementation.buffer.clone());
            self.0
                .referenced_objects
                .shared_device_memory_allocations
                .push(device_buffer_implementation.device_memory.clone());
            device.vkCmdPipelineBarrier.unwrap()(
                command_buffer.command_buffer,
                api::VK_PIPELINE_STAGE_TRANSFER_BIT,
                api::VK_PIPELINE_STAGE_VERTEX_INPUT_BIT,
                0,
                0,
                null(),
                1,
                &api::VkBufferMemoryBarrier {
                    sType: api::VK_STRUCTURE_TYPE_BUFFER_MEMORY_BARRIER,
                    pNext: null(),
                    srcAccessMask: api::VK_ACCESS_TRANSFER_WRITE_BIT,
                    dstAccessMask: api::VK_ACCESS_INDEX_READ_BIT,
                    srcQueueFamilyIndex: self.0.command_buffer.queue_family_index,
                    dstQueueFamilyIndex: self.0.command_buffer.queue_family_index,
                    buffer: device_buffer_implementation.buffer.buffer,
                    offset: 0,
                    size: api::VK_WHOLE_SIZE as u64,
                },
                0,
                null(),
            );
        }
        Ok(device_buffer)
    }
    fn finish(self) -> Result<VulkanLoaderCommandBuffer> {
        let mut retval = self.0;
        retval.command_buffer = unsafe { retval.command_buffer.finish() }?;
        Ok(retval)
    }
}

struct VulkanRenderCommandBufferState {
    command_buffer: CommandBufferWrapper,
    referenced_objects: CommandBufferReferencedObjects,
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
                referenced_objects: Default::default(),
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
