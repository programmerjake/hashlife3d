use super::{
    api, get_mut_vulkan_device_index_buffer_implementation,
    get_mut_vulkan_device_vertex_buffer_implementation,
    into_vulkan_staging_index_buffer_implementation,
    into_vulkan_staging_vertex_buffer_implementation, null_or_zero, BufferWrapper,
    DeviceMemoryPoolAllocation, DeviceWrapper, FenceState, FenceWrapper, Result, SemaphoreWrapper,
    VulkanDevice, VulkanDeviceIndexBuffer, VulkanDeviceVertexBuffer, VulkanError,
    VulkanStagingIndexBuffer, VulkanStagingIndexBufferImplementation, VulkanStagingVertexBuffer,
    VulkanStagingVertexBufferImplementation, COLOR_ATTACHEMENT_INDEX, DEPTH_ATTACHEMENT_INDEX,
};
use renderer::{
    math, CommandBuffer, IndexBufferElement, LoaderCommandBufferBuilder,
    RenderCommandBufferBuilder, VertexBufferElement,
};
use std::any::Any;
use std::cmp;
use std::mem;
use std::ptr::{null, null_mut};
use std::sync::atomic::*;
use std::sync::Arc;
use std::u64;

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

pub struct CommandBufferReferencedObjects {
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

trait GenericCommandBuffer {
    fn get_command_buffer(&self) -> &CommandBufferWrapper;
    fn get_submit_tracker(&self) -> &CommandBufferSubmitTracker;
    fn get_referenced_objects(&self) -> &CommandBufferReferencedObjects;
}

pub struct VulkanLoaderCommandBuffer {
    command_buffer: CommandBufferWrapper,
    submit_tracker: CommandBufferSubmitTracker,
    referenced_objects: CommandBufferReferencedObjects,
}

impl GenericCommandBuffer for VulkanLoaderCommandBuffer {
    fn get_command_buffer(&self) -> &CommandBufferWrapper {
        &self.command_buffer
    }
    fn get_submit_tracker(&self) -> &CommandBufferSubmitTracker {
        &self.submit_tracker
    }
    fn get_referenced_objects(&self) -> &CommandBufferReferencedObjects {
        &self.referenced_objects
    }
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
    submit_tracker: CommandBufferSubmitTracker,
}

#[derive(Clone)]
pub struct VulkanRenderCommandBuffer(Arc<VulkanRenderCommandBufferState>);

impl GenericCommandBuffer for VulkanRenderCommandBuffer {
    fn get_command_buffer(&self) -> &CommandBufferWrapper {
        &self.0.command_buffer
    }
    fn get_submit_tracker(&self) -> &CommandBufferSubmitTracker {
        &self.0.submit_tracker
    }
    fn get_referenced_objects(&self) -> &CommandBufferReferencedObjects {
        &self.0.referenced_objects
    }
}

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
                )?.begin(
                    api::VK_COMMAND_BUFFER_USAGE_RENDER_PASS_CONTINUE_BIT
                        | api::VK_COMMAND_BUFFER_USAGE_SIMULTANEOUS_USE_BIT,
                    unimplemented!(),
                )?,
                submit_tracker: CommandBufferSubmitTracker::new(),
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

fn submit_loader_command_buffers_with_semaphores(
    vulkan_device: &mut VulkanDevice,
    wait_semaphores: &[(SemaphoreWrapper, api::VkPipelineStageFlags)],
    loader_command_buffers: &mut Vec<VulkanLoaderCommandBuffer>,
    signal_semaphores: &[SemaphoreWrapper],
) -> Result<()> {
    let device = &vulkan_device.device_reference.device;
    let mut command_buffers = Vec::with_capacity(loader_command_buffers.len());
    let mut referenced_objects: Vec<Box<Any>> = Vec::new();
    for command_buffer in loader_command_buffers.drain(..) {
        for required_command_buffer in &command_buffer.referenced_objects.required_command_buffers {
            required_command_buffer.assert_submitted();
        }
        command_buffers.push(command_buffer.command_buffer.command_buffer);
        unsafe {
            command_buffer.submit_tracker.set_submitted();
        }
        referenced_objects.push(Box::new(command_buffer));
    }
    let fence = FenceWrapper::new(device.clone(), FenceState::Unsignaled)?;
    let wait_destination_stage_masks: Vec<_> = wait_semaphores.iter().map(|v| v.1).collect();
    let wait_semaphores: Vec<_> = wait_semaphores.iter().map(|v| v.0.semaphore).collect();
    let signal_semaphores: Vec<_> = signal_semaphores.iter().map(|v| v.semaphore).collect();
    match unsafe {
        device.vkQueueSubmit.unwrap()(
            vulkan_device.render_queue,
            1,
            &api::VkSubmitInfo {
                sType: api::VK_STRUCTURE_TYPE_SUBMIT_INFO,
                pNext: null(),
                waitSemaphoreCount: wait_semaphores.len() as u32,
                pWaitSemaphores: wait_semaphores.as_ptr(),
                pWaitDstStageMask: wait_destination_stage_masks.as_ptr(),
                commandBufferCount: command_buffers.len() as u32,
                pCommandBuffers: command_buffers.as_ptr(),
                signalSemaphoreCount: signal_semaphores.len() as u32,
                pSignalSemaphores: signal_semaphores.as_ptr(),
            },
            fence.fence,
        )
    } {
        api::VK_SUCCESS => {
            vulkan_device
                .in_progress_operations
                .push_back((fence, referenced_objects));
            Ok(())
        }
        result => Err(VulkanError::VulkanError(result)),
    }
}

pub fn submit_loader_command_buffers(
    vulkan_device: &mut VulkanDevice,
    loader_command_buffers: &mut Vec<VulkanLoaderCommandBuffer>,
) -> Result<()> {
    vulkan_device.free_finished_objects()?;
    if loader_command_buffers.is_empty() {
        return Ok(());
    }
    submit_loader_command_buffers_with_semaphores(vulkan_device, &[], loader_command_buffers, &[])
}

pub unsafe fn render_frame(
    vulkan_device: &mut VulkanDevice,
    clear_color: math::Vec4<f32>,
    loader_command_buffers: &mut Vec<VulkanLoaderCommandBuffer>,
    render_command_buffers: &[VulkanRenderCommandBuffer],
) -> Result<()> {
    vulkan_device.free_finished_objects()?;
    let swapchain = match vulkan_device.swapchain.clone() {
        Some(swapchain) => swapchain,
        None => return submit_loader_command_buffers(vulkan_device, loader_command_buffers),
    };
    let graphics_pipeline = vulkan_device.graphics_pipeline.as_ref().unwrap();
    let device = &vulkan_device.device_reference.device;
    let image_acquired_semaphore = SemaphoreWrapper::new(device.clone())?;
    let image_acquired_fence = FenceWrapper::new(device.clone(), FenceState::Unsignaled)?;
    let mut image_index = 0;
    let image_index = match device.vkAcquireNextImageKHR.unwrap()(
        device.device,
        swapchain.swapchain.swapchain,
        u64::MAX,
        image_acquired_semaphore.semaphore,
        image_acquired_fence.fence,
        &mut image_index,
    ) {
        api::VK_SUCCESS => Ok(image_index as usize),
        result => Err(VulkanError::VulkanError(result)),
    }?;
    let mut command_buffers = Vec::with_capacity(loader_command_buffers.len() + 1);
    let mut referenced_objects: Vec<Box<Any>> = Vec::new();
    for command_buffer in loader_command_buffers.drain(..) {
        for required_command_buffer in &command_buffer.referenced_objects.required_command_buffers {
            required_command_buffer.assert_submitted();
        }
        command_buffers.push(command_buffer.command_buffer.command_buffer);
        command_buffer.submit_tracker.set_submitted();
        referenced_objects.push(Box::new(command_buffer));
    }
    let render_command_buffer = CommandBufferWrapper::new(
        device,
        vulkan_device
            .surface_state
            .as_ref()
            .unwrap()
            .render_queue_index,
        api::VK_COMMAND_BUFFER_LEVEL_PRIMARY,
    )?.begin(api::VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT, None)?;
    device.vkCmdSetViewport.unwrap()(
        render_command_buffer.command_buffer,
        0,
        1,
        &api::VkViewport {
            x: 0.0,
            y: 0.0,
            width: swapchain.dimensions.0 as f32,
            height: swapchain.dimensions.1 as f32,
            minDepth: 0.0,
            maxDepth: 1.0,
        },
    );
    device.vkCmdSetScissor.unwrap()(
        render_command_buffer.command_buffer,
        0,
        1,
        &api::VkRect2D {
            offset: api::VkOffset2D { x: 0, y: 0 },
            extent: api::VkExtent2D {
                width: swapchain.dimensions.0,
                height: swapchain.dimensions.1,
            },
        },
    );
    let mut clear_values: [api::VkClearValue; 2] = mem::zeroed();
    clear_values[DEPTH_ATTACHEMENT_INDEX] = api::VkClearValue {
        depthStencil: api::VkClearDepthStencilValue {
            depth: 1.0,
            stencil: 0,
        },
    };
    clear_values[COLOR_ATTACHEMENT_INDEX] = api::VkClearValue {
        color: api::VkClearColorValue {
            float32: [clear_color.x, clear_color.y, clear_color.z, clear_color.w],
        },
    };
    device.vkCmdBeginRenderPass.unwrap()(
        render_command_buffer.command_buffer,
        &api::VkRenderPassBeginInfo {
            sType: api::VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO,
            pNext: null(),
            renderPass: graphics_pipeline.render_pass.render_pass,
            framebuffer: swapchain.framebuffers[image_index].framebuffer,
            renderArea: api::VkRect2D {
                offset: api::VkOffset2D { x: 0, y: 0 },
                extent: api::VkExtent2D {
                    width: swapchain.dimensions.0,
                    height: swapchain.dimensions.1,
                },
            },
            clearValueCount: clear_values.len() as u32,
            pClearValues: clear_values.as_ptr(),
        },
        api::VK_SUBPASS_CONTENTS_SECONDARY_COMMAND_BUFFERS,
    );
    let mut render_pass_command_buffers = Vec::with_capacity(render_command_buffers.len());
    for command_buffer in render_command_buffers {
        for required_command_buffer in &command_buffer.0.referenced_objects.required_command_buffers
        {
            required_command_buffer.assert_submitted();
        }
        render_pass_command_buffers.push(command_buffer.0.command_buffer.command_buffer);
        command_buffer.0.submit_tracker.set_submitted();
        referenced_objects.push(Box::new(command_buffer.clone()));
    }
    if !render_pass_command_buffers.is_empty() {
        device.vkCmdExecuteCommands.unwrap()(
            render_command_buffer.command_buffer,
            render_pass_command_buffers.len() as u32,
            render_pass_command_buffers.as_ptr(),
        );
    }
    device.vkCmdEndRenderPass.unwrap()(render_command_buffer.command_buffer);
    let render_command_buffer = render_command_buffer.finish()?;
    command_buffers.push(render_command_buffer.command_buffer);
    referenced_objects.push(Box::new(render_command_buffer));
    let render_completed_semaphore = if vulkan_device.in_progress_present_semaphores.len()
        >= cmp::max(16, 2 * swapchain.framebuffers.len())
    {
        vulkan_device
            .in_progress_present_semaphores
            .pop_front()
            .unwrap()
    } else {
        SemaphoreWrapper::new(device.clone())?
    };
    let fence = FenceWrapper::new(device.clone(), FenceState::Unsignaled)?;
    match device.vkQueueSubmit.unwrap()(
        vulkan_device.render_queue,
        1,
        &api::VkSubmitInfo {
            sType: api::VK_STRUCTURE_TYPE_SUBMIT_INFO,
            pNext: null(),
            waitSemaphoreCount: 1,
            pWaitSemaphores: &image_acquired_semaphore.semaphore,
            pWaitDstStageMask: &api::VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT,
            commandBufferCount: command_buffers.len() as u32,
            pCommandBuffers: command_buffers.as_ptr(),
            signalSemaphoreCount: 1,
            pSignalSemaphores: &render_completed_semaphore.semaphore,
        },
        fence.fence,
    ) {
        api::VK_SUCCESS => {}
        result => return Err(VulkanError::VulkanError(result)),
    }
    referenced_objects.push(Box::new(image_acquired_semaphore));
    vulkan_device
        .in_progress_operations
        .push_back((fence, referenced_objects));
    let mut present_result = api::VK_SUCCESS;
    match device.vkQueuePresentKHR.unwrap()(
        vulkan_device.present_queue,
        &api::VkPresentInfoKHR {
            sType: api::VK_STRUCTURE_TYPE_PRESENT_INFO_KHR,
            pNext: null(),
            waitSemaphoreCount: 1,
            pWaitSemaphores: &render_completed_semaphore.semaphore,
            swapchainCount: 1,
            pSwapchains: &swapchain.swapchain.swapchain,
            pImageIndices: &(image_index as u32),
            pResults: &mut present_result,
        },
    ) {
        api::VK_SUCCESS => {}
        result => return Err(VulkanError::VulkanError(result)),
    }
    match present_result {
        api::VK_SUCCESS => {}
        result => return Err(VulkanError::VulkanError(result)),
    }
    vulkan_device
        .in_progress_present_semaphores
        .push_back(render_completed_semaphore);
    match device.vkWaitForFences.unwrap()(
        device.device,
        1,
        &image_acquired_fence.fence,
        api::VK_FALSE,
        u64::MAX,
    ) {
        api::VK_SUCCESS => Ok(()),
        result => return Err(VulkanError::VulkanError(result)),
    }
}
