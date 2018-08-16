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
use super::{
    api, create_device_image_set, get_mut_vulkan_device_index_buffer_implementation,
    get_mut_vulkan_device_vertex_buffer_implementation,
    into_vulkan_device_image_set_implementation, into_vulkan_device_index_buffer_implementation,
    into_vulkan_device_vertex_buffer_implementation, into_vulkan_staging_image_set_implementation,
    into_vulkan_staging_index_buffer_implementation,
    into_vulkan_staging_vertex_buffer_implementation, null_or_zero, set_push_constants,
    set_push_constants_initial_transform, BufferWrapper, DescriptorSetWrapper,
    DeviceMemoryPoolAllocation, DeviceWrapper, FenceState, FenceWrapper, GraphicsPipelineWrapper,
    ImageViewWrapper, PipelineLayoutWrapper, PushConstants, RenderPassWrapper, Result,
    SemaphoreWrapper, VulkanDevice, VulkanDeviceImageSet, VulkanDeviceImageSetImplementation,
    VulkanDeviceIndexBuffer, VulkanDeviceIndexBufferImplementation, VulkanDeviceVertexBuffer,
    VulkanDeviceVertexBufferImplementation, VulkanError, VulkanStagingImageSet,
    VulkanStagingImageSetImplementation, VulkanStagingIndexBuffer,
    VulkanStagingIndexBufferImplementation, VulkanStagingVertexBuffer,
    VulkanStagingVertexBufferImplementation, COLOR_ATTACHEMENT_INDEX, DEPTH_ATTACHEMENT_INDEX,
    SAMPLERS_DESCRIPTOR_SET_INDEX,
};
use renderer::{
    image::Pixel, math, CommandBuffer, Device, DeviceIndexBuffer, DeviceVertexBuffer,
    IndexBufferElement, LoaderCommandBufferBuilder, RenderCommandBufferBuilder,
    RenderCommandBufferGroup, VertexBufferElement,
};
use sdl;
use std::any::Any;
use std::cmp;
use std::mem;
use std::ptr::{null, null_mut};
use std::sync::atomic::*;
use std::sync::{Arc, Mutex};
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
    shared_image_view_vecs: Vec<Arc<Vec<ImageViewWrapper>>>,
    shared_descriptor_sets: Vec<Arc<DescriptorSetWrapper>>,
}

impl Default for CommandBufferReferencedObjects {
    fn default() -> Self {
        Self {
            required_command_buffers: Vec::new(),
            device_memory_allocations: Vec::new(),
            shared_device_memory_allocations: Vec::new(),
            buffers: Vec::new(),
            shared_buffers: Vec::new(),
            shared_image_view_vecs: Vec::new(),
            shared_descriptor_sets: Vec::new(),
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
    type StagingImageSet = VulkanStagingImageSet;
    type DeviceImageSet = VulkanDeviceImageSet;
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
                    size: device_buffer_implementation.element_count as u64
                        * mem::size_of::<VertexBufferElement>() as u64,
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
                    size: device_buffer_implementation.element_count as u64
                        * mem::size_of::<IndexBufferElement>() as u64,
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
    fn copy_image_set_to_device(
        &mut self,
        staging_image_set: VulkanStagingImageSet,
    ) -> Result<VulkanDeviceImageSet> {
        let command_buffer = &self.0.command_buffer;
        let device = &command_buffer.command_pool.device;
        let VulkanStagingImageSetImplementation {
            buffer: staging_buffer,
            buffer_allocation: staging_buffer_allocation,
            mut device_image_set,
            mapped_memory: _,
        } = into_vulkan_staging_image_set_implementation(staging_image_set);
        unsafe {
            device_image_set.submit_tracker = Some(self.0.submit_tracker.clone());
            let mut start_image_memory_barriers = Vec::new();
            let mut end_image_memory_barriers = Vec::new();
            for image in &*device_image_set.images {
                start_image_memory_barriers.push(api::VkImageMemoryBarrier {
                    sType: api::VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER,
                    pNext: null(),
                    srcAccessMask: 0,
                    dstAccessMask: api::VK_ACCESS_TRANSFER_WRITE_BIT,
                    oldLayout: api::VK_IMAGE_LAYOUT_UNDEFINED,
                    newLayout: api::VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
                    srcQueueFamilyIndex: self.0.command_buffer.queue_family_index,
                    dstQueueFamilyIndex: self.0.command_buffer.queue_family_index,
                    image: image.image.image,
                    subresourceRange: api::VkImageSubresourceRange {
                        aspectMask: api::VK_IMAGE_ASPECT_COLOR_BIT,
                        baseMipLevel: 0,
                        levelCount: 1,
                        baseArrayLayer: 0,
                        layerCount: api::VK_REMAINING_ARRAY_LAYERS as u32,
                    },
                });
                end_image_memory_barriers.push(api::VkImageMemoryBarrier {
                    sType: api::VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER,
                    pNext: null(),
                    srcAccessMask: api::VK_ACCESS_TRANSFER_WRITE_BIT,
                    dstAccessMask: api::VK_ACCESS_SHADER_READ_BIT,
                    oldLayout: api::VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
                    newLayout: api::VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL,
                    srcQueueFamilyIndex: self.0.command_buffer.queue_family_index,
                    dstQueueFamilyIndex: self.0.command_buffer.queue_family_index,
                    image: image.image.image,
                    subresourceRange: api::VkImageSubresourceRange {
                        aspectMask: api::VK_IMAGE_ASPECT_COLOR_BIT,
                        baseMipLevel: 0,
                        levelCount: 1,
                        baseArrayLayer: 0,
                        layerCount: api::VK_REMAINING_ARRAY_LAYERS as u32,
                    },
                });
            }
            device.vkCmdPipelineBarrier.unwrap()(
                command_buffer.command_buffer,
                api::VK_PIPELINE_STAGE_HOST_BIT,
                api::VK_PIPELINE_STAGE_TRANSFER_BIT,
                0,
                0,
                null(),
                0,
                null(),
                start_image_memory_barriers.len() as u32,
                start_image_memory_barriers.as_ptr(),
            );
            self.0
                .referenced_objects
                .shared_image_view_vecs
                .push(device_image_set.images.clone());
            let width = device_image_set.width as usize;
            let height = device_image_set.height as usize;
            let image_size = width * height * mem::size_of::<Pixel>();
            let mut buffer_image_copy_structs = Vec::new();
            for (image_index, image) in device_image_set.images.iter().enumerate() {
                if image_index as u32 >= device_image_set.valid_image_count {
                    device.vkCmdClearColorImage.unwrap()(
                        command_buffer.command_buffer,
                        image.image.image,
                        api::VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
                        &api::VkClearColorValue { float32: [0.0; 4] },
                        1,
                        &api::VkImageSubresourceRange {
                            aspectMask: api::VK_IMAGE_ASPECT_COLOR_BIT,
                            baseMipLevel: 0,
                            levelCount: 1,
                            baseArrayLayer: 0,
                            layerCount: 1,
                        },
                    );
                } else {
                    buffer_image_copy_structs.clear();
                    let image_layer_count =
                        if image_index + 1 == device_image_set.valid_image_count as usize {
                            device_image_set.last_image_layer_count
                        } else {
                            device_image_set.image_layer_count
                        };
                    for layer in 0..image_layer_count {
                        buffer_image_copy_structs.push(api::VkBufferImageCopy {
                            bufferOffset: (image_size
                                * (image_index * device_image_set.image_layer_count as usize
                                    + layer as usize))
                                as api::VkDeviceSize,
                            bufferRowLength: 0,
                            bufferImageHeight: 0,
                            imageSubresource: api::VkImageSubresourceLayers {
                                aspectMask: api::VK_IMAGE_ASPECT_COLOR_BIT,
                                mipLevel: 0,
                                baseArrayLayer: layer,
                                layerCount: 1,
                            },
                            imageOffset: api::VkOffset3D { x: 0, y: 0, z: 0 },
                            imageExtent: api::VkExtent3D {
                                width: width as u32,
                                height: height as u32,
                                depth: 1,
                            },
                        });
                    }
                    device.vkCmdCopyBufferToImage.unwrap()(
                        command_buffer.command_buffer,
                        staging_buffer.buffer,
                        image.image.image,
                        api::VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
                        buffer_image_copy_structs.len() as u32,
                        buffer_image_copy_structs.as_ptr(),
                    );
                }
            }
            self.0.referenced_objects.buffers.push(staging_buffer);
            self.0
                .referenced_objects
                .device_memory_allocations
                .push(staging_buffer_allocation);
            device.vkCmdPipelineBarrier.unwrap()(
                command_buffer.command_buffer,
                api::VK_PIPELINE_STAGE_TRANSFER_BIT,
                api::VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT,
                0,
                0,
                null(),
                0,
                null(),
                end_image_memory_barriers.len() as u32,
                end_image_memory_barriers.as_ptr(),
            );
        }
        Ok(create_device_image_set(device_image_set))
    }
    fn finish(self) -> Result<VulkanLoaderCommandBuffer> {
        let mut retval = self.0;
        retval.command_buffer = unsafe { retval.command_buffer.finish() }?;
        Ok(retval)
    }
}

#[derive(Clone)]
enum RenderCommand {
    SetImageSet {
        image_set: VulkanDeviceImageSet,
    },
    SetBuffers {
        vertex_buffer: VulkanDeviceVertexBuffer,
        index_buffer: VulkanDeviceIndexBuffer,
    },
    SetInitialTransform {
        transform: math::Mat4<f32>,
    },
    Draw {
        index_count: u32,
        first_index: u32,
        vertex_offset: u32,
    },
}

#[derive(Copy, Clone)]
struct VulkanRenderCommandBufferGeneratedStateKey {
    dimensions: (u32, u32),
    final_transform: math::Mat4<f32>,
}

impl Eq for VulkanRenderCommandBufferGeneratedStateKey {}

impl PartialEq for VulkanRenderCommandBufferGeneratedStateKey {
    fn eq(&self, rhs: &Self) -> bool {
        use self::math::Mappable;
        let into_compare_key = |v: &Self| (v.dimensions, v.final_transform.map(|v| v.to_bits()));
        into_compare_key(self) == into_compare_key(rhs)
    }
}

struct VulkanRenderCommandBufferGeneratedState {
    key: VulkanRenderCommandBufferGeneratedStateKey,
    command_buffer: CommandBufferWrapper,
    referenced_objects: CommandBufferReferencedObjects,
    submit_tracker: CommandBufferSubmitTracker,
}

struct VulkanRenderCommandBufferState {
    render_commands: Vec<RenderCommand>,
    device: Arc<DeviceWrapper>,
    queue_family_index: u32,
    render_pass: Arc<RenderPassWrapper>,
    pipeline_layout: Arc<PipelineLayoutWrapper>,
    graphics_pipeline: Arc<GraphicsPipelineWrapper>,
    generated_state: Option<Arc<VulkanRenderCommandBufferGeneratedState>>,
}

impl VulkanRenderCommandBufferState {
    fn new(
        render_commands: Vec<RenderCommand>,
        device: Arc<DeviceWrapper>,
        queue_family_index: u32,
        render_pass: Arc<RenderPassWrapper>,
        pipeline_layout: Arc<PipelineLayoutWrapper>,
        graphics_pipeline: Arc<GraphicsPipelineWrapper>,
    ) -> Self {
        Self {
            render_commands: render_commands,
            device: device,
            queue_family_index: queue_family_index,
            render_pass: render_pass,
            pipeline_layout: pipeline_layout,
            graphics_pipeline: graphics_pipeline,
            generated_state: None,
        }
    }
    unsafe fn generate_state(
        &mut self,
        key: VulkanRenderCommandBufferGeneratedStateKey,
    ) -> Result<Arc<VulkanRenderCommandBufferGeneratedState>> {
        if let Some(generated_state) = &self.generated_state {
            if generated_state.key == key {
                return Ok(generated_state.clone());
            }
        }
        self.generated_state.take();
        let command_buffer = CommandBufferWrapper::new(
            &self.device,
            self.queue_family_index,
            api::VK_COMMAND_BUFFER_LEVEL_SECONDARY,
        )?.begin(
            api::VK_COMMAND_BUFFER_USAGE_RENDER_PASS_CONTINUE_BIT
                | api::VK_COMMAND_BUFFER_USAGE_SIMULTANEOUS_USE_BIT,
            Some(&api::VkCommandBufferInheritanceInfo {
                sType: api::VK_STRUCTURE_TYPE_COMMAND_BUFFER_INHERITANCE_INFO,
                pNext: null(),
                renderPass: self.render_pass.render_pass,
                subpass: 0,
                framebuffer: null_or_zero(),
                occlusionQueryEnable: api::VK_FALSE,
                queryFlags: 0,
                pipelineStatistics: 0,
            }),
        )?;
        self.device.vkCmdSetViewport.unwrap()(
            command_buffer.command_buffer,
            0,
            1,
            &api::VkViewport {
                x: 0.0,
                y: 0.0,
                width: key.dimensions.0 as f32,
                height: key.dimensions.1 as f32,
                minDepth: 0.0,
                maxDepth: 1.0,
            },
        );
        self.device.vkCmdSetScissor.unwrap()(
            command_buffer.command_buffer,
            0,
            1,
            &api::VkRect2D {
                offset: api::VkOffset2D { x: 0, y: 0 },
                extent: api::VkExtent2D {
                    width: key.dimensions.0,
                    height: key.dimensions.1,
                },
            },
        );
        self.device.vkCmdBindPipeline.unwrap()(
            command_buffer.command_buffer,
            api::VK_PIPELINE_BIND_POINT_GRAPHICS,
            self.graphics_pipeline.pipeline,
        );
        set_push_constants(
            &self.device,
            command_buffer.command_buffer,
            self.pipeline_layout.pipeline_layout,
            api::VK_SHADER_STAGE_VERTEX_BIT,
            PushConstants {
                initial_transform: math::Mat4::identity().into(),
                final_transform: key.final_transform.into(),
            },
        );
        let submit_tracker = CommandBufferSubmitTracker::new();
        let mut referenced_objects: CommandBufferReferencedObjects = Default::default();
        for render_command in &self.render_commands {
            match render_command.clone() {
                RenderCommand::SetImageSet { image_set } => {
                    let VulkanDeviceImageSetImplementation {
                        images,
                        submit_tracker,
                        width: _,
                        height: _,
                        total_layer_count: _,
                        image_layer_count: _,
                        last_image_layer_count: _,
                        valid_image_count: _,
                        descriptor_set,
                    } = into_vulkan_device_image_set_implementation(image_set);
                    self.device.vkCmdBindDescriptorSets.unwrap()(
                        command_buffer.command_buffer,
                        api::VK_PIPELINE_BIND_POINT_GRAPHICS,
                        self.pipeline_layout.pipeline_layout,
                        SAMPLERS_DESCRIPTOR_SET_INDEX,
                        1,
                        &descriptor_set.descriptor_set,
                        0,
                        null(),
                    );
                    referenced_objects.shared_image_view_vecs.push(images);
                    referenced_objects
                        .required_command_buffers
                        .push(submit_tracker.unwrap());
                    referenced_objects
                        .shared_descriptor_sets
                        .push(descriptor_set);
                }
                RenderCommand::SetBuffers {
                    vertex_buffer,
                    index_buffer,
                } => {
                    let VulkanDeviceVertexBufferImplementation {
                        buffer: vertex_buffer,
                        device_memory: vertex_device_memory,
                        submit_tracker: vertex_submit_tracker,
                        element_count: _,
                    } = into_vulkan_device_vertex_buffer_implementation(vertex_buffer);
                    self.device.vkCmdBindVertexBuffers.unwrap()(
                        command_buffer.command_buffer,
                        0,
                        1,
                        &vertex_buffer.buffer,
                        &0,
                    );
                    referenced_objects.shared_buffers.push(vertex_buffer);
                    referenced_objects
                        .shared_device_memory_allocations
                        .push(vertex_device_memory);
                    referenced_objects
                        .required_command_buffers
                        .push(vertex_submit_tracker.unwrap());
                    let VulkanDeviceIndexBufferImplementation {
                        buffer: index_buffer,
                        device_memory: index_device_memory,
                        submit_tracker: index_submit_tracker,
                        element_count: _,
                    } = into_vulkan_device_index_buffer_implementation(index_buffer);
                    self.device.vkCmdBindIndexBuffer.unwrap()(
                        command_buffer.command_buffer,
                        index_buffer.buffer,
                        0,
                        api::VK_INDEX_TYPE_UINT16,
                    );
                    referenced_objects.shared_buffers.push(index_buffer);
                    referenced_objects
                        .shared_device_memory_allocations
                        .push(index_device_memory);
                    referenced_objects
                        .required_command_buffers
                        .push(index_submit_tracker.unwrap());
                }
                RenderCommand::SetInitialTransform { transform } => {
                    set_push_constants_initial_transform(
                        &self.device,
                        command_buffer.command_buffer,
                        self.pipeline_layout.pipeline_layout,
                        api::VK_SHADER_STAGE_VERTEX_BIT,
                        transform.into(),
                    );
                }
                RenderCommand::Draw {
                    index_count,
                    first_index,
                    vertex_offset,
                } => {
                    self.device.vkCmdDrawIndexed.unwrap()(
                        command_buffer.command_buffer,
                        index_count,
                        1,
                        first_index,
                        vertex_offset as i32,
                        0,
                    );
                }
            }
        }
        let command_buffer = command_buffer.finish()?;
        let retval = Arc::new(VulkanRenderCommandBufferGeneratedState {
            key: key,
            command_buffer: command_buffer,
            referenced_objects: referenced_objects,
            submit_tracker: submit_tracker,
        });
        self.generated_state = Some(retval.clone());
        Ok(retval)
    }
}

#[derive(Clone)]
pub struct VulkanRenderCommandBuffer(Arc<Mutex<VulkanRenderCommandBufferState>>);

unsafe impl Send for VulkanRenderCommandBuffer {}

impl CommandBuffer for VulkanRenderCommandBuffer {}

pub struct VulkanRenderCommandBufferBuilder {
    render_commands: Vec<RenderCommand>,
    device: Arc<DeviceWrapper>,
    queue_family_index: u32,
    render_pass: Arc<RenderPassWrapper>,
    pipeline_layout: Arc<PipelineLayoutWrapper>,
    graphics_pipeline: Arc<GraphicsPipelineWrapper>,
    did_set_initial_transform: bool,
    index_buffer_length: usize,
    vertex_buffer_length: usize,
    did_set_image_set: bool,
}

impl VulkanRenderCommandBufferBuilder {
    pub unsafe fn new(
        device: Arc<DeviceWrapper>,
        queue_family_index: u32,
        render_pass: Arc<RenderPassWrapper>,
        pipeline_layout: Arc<PipelineLayoutWrapper>,
        graphics_pipeline: Arc<GraphicsPipelineWrapper>,
    ) -> Self {
        Self {
            render_commands: Vec::new(),
            device: device,
            queue_family_index: queue_family_index,
            render_pass: render_pass,
            pipeline_layout: pipeline_layout,
            graphics_pipeline: graphics_pipeline,
            did_set_initial_transform: false,
            index_buffer_length: 0,
            vertex_buffer_length: 0,
            did_set_image_set: false,
        }
    }
}

impl RenderCommandBufferBuilder for VulkanRenderCommandBufferBuilder {
    type Error = VulkanError;
    type CommandBuffer = VulkanRenderCommandBuffer;
    type DeviceVertexBuffer = VulkanDeviceVertexBuffer;
    type DeviceIndexBuffer = VulkanDeviceIndexBuffer;
    type DeviceImageSet = VulkanDeviceImageSet;
    fn set_buffers(
        &mut self,
        vertex_buffer: VulkanDeviceVertexBuffer,
        index_buffer: VulkanDeviceIndexBuffer,
    ) {
        self.vertex_buffer_length = vertex_buffer.len();
        self.index_buffer_length = index_buffer.len();
        self.render_commands.push(RenderCommand::SetBuffers {
            vertex_buffer: vertex_buffer,
            index_buffer: index_buffer,
        });
    }
    fn set_image_set(&mut self, image_set: VulkanDeviceImageSet) {
        self.render_commands.push(RenderCommand::SetImageSet {
            image_set: image_set,
        });
        self.did_set_image_set = true;
    }
    fn set_initial_transform(&mut self, transform: math::Mat4<f32>) {
        self.did_set_initial_transform = true;
        self.render_commands
            .push(RenderCommand::SetInitialTransform {
                transform: transform,
            });
    }
    fn draw(&mut self, index_count: u32, first_index: u32, vertex_offset: u32) {
        assert!(index_count as usize <= self.index_buffer_length);
        assert!(index_count as usize + first_index as usize <= self.index_buffer_length);
        assert!((vertex_offset as usize) < self.vertex_buffer_length);
        assert!(index_count % 3 == 0, "must be whole number of triangles");
        assert!(self.did_set_image_set);
        if index_count > 0 {
            if !self.did_set_initial_transform {
                self.did_set_initial_transform = true;
                self.set_initial_transform(math::Mat4::identity());
            }
            self.render_commands.push(RenderCommand::Draw {
                index_count: index_count,
                first_index: first_index,
                vertex_offset: vertex_offset,
            });
        }
    }
    fn finish(self) -> Result<VulkanRenderCommandBuffer> {
        Ok(VulkanRenderCommandBuffer(Arc::new(Mutex::new(
            VulkanRenderCommandBufferState::new(
                self.render_commands,
                self.device,
                self.queue_family_index,
                self.render_pass,
                self.pipeline_layout,
                self.graphics_pipeline,
            ),
        ))))
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
    match unsafe {
        device.vkQueueSubmit.unwrap()(
            vulkan_device.render_queue,
            1,
            &api::VkSubmitInfo {
                sType: api::VK_STRUCTURE_TYPE_SUBMIT_INFO,
                pNext: null(),
                waitSemaphoreCount: 0,
                pWaitSemaphores: null(),
                pWaitDstStageMask: null(),
                commandBufferCount: command_buffers.len() as u32,
                pCommandBuffers: command_buffers.as_ptr(),
                signalSemaphoreCount: 0,
                pSignalSemaphores: null(),
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

pub unsafe fn render_frame(
    vulkan_device: &mut VulkanDevice,
    clear_color: math::Vec4<f32>,
    loader_command_buffers: &mut Vec<VulkanLoaderCommandBuffer>,
    render_command_buffer_groups: &[RenderCommandBufferGroup<VulkanRenderCommandBuffer>],
) -> Result<()> {
    vulkan_device.free_finished_objects()?;
    let mut sdl_dimensions = (0, 0);
    sdl::api::SDL_Vulkan_GetDrawableSize(
        vulkan_device.get_window().get(),
        &mut sdl_dimensions.0,
        &mut sdl_dimensions.1,
    );
    let mut sdl_dimensions = match sdl_dimensions {
        (0, _) | (_, 0) => None,
        sdl_dimensions => Some((sdl_dimensions.0 as u32, sdl_dimensions.1 as u32)),
    };
    let swapchain = match vulkan_device.swapchain.clone() {
        Some(swapchain) => swapchain,
        None => return submit_loader_command_buffers(vulkan_device, loader_command_buffers),
    };
    let graphics_pipeline = vulkan_device
        .device_reference
        .graphics_pipeline
        .as_ref()
        .unwrap();
    let device = &vulkan_device.device_reference.device;
    // query dimensions from Vulkan to make vulkan layers happy
    let mut physical_device_surface_capabilities = mem::zeroed();
    let vulkan_dimensions = match device
        .instance
        .vkGetPhysicalDeviceSurfaceCapabilitiesKHR
        .unwrap()(
        vulkan_device
            .surface_state
            .as_ref()
            .unwrap()
            .physical_device,
        vulkan_device
            .surface_state
            .as_ref()
            .unwrap()
            .surface
            .surface,
        &mut physical_device_surface_capabilities,
    ) {
        api::VK_SUCCESS => {
            let api::VkExtent2D { width, height } =
                physical_device_surface_capabilities.currentExtent;
            (width, height)
        }
        result => return Err(VulkanError::VulkanError(result)),
    };
    if vulkan_dimensions != (0xFFFFFFFF, 0xFFFFFFFF) {
        sdl_dimensions = match vulkan_dimensions {
            (0, _) | (_, 0) => None,
            _ => Some(vulkan_dimensions),
        };
    }
    let image_acquired_semaphore = SemaphoreWrapper::new(device.clone())?;
    let image_acquired_fence = FenceWrapper::new(device.clone(), FenceState::Unsignaled)?;
    let mut image_index = 0;
    struct AcquireImageResults {
        image_index: Option<usize>,
        need_new_swapchain: bool,
        image_acquired_semaphore: Option<SemaphoreWrapper>,
        image_acquired_fence: Option<FenceWrapper>,
    }
    let AcquireImageResults {
        image_index,
        need_new_swapchain,
        image_acquired_semaphore,
        image_acquired_fence,
    } = match device.vkAcquireNextImageKHR.unwrap()(
        device.device,
        swapchain.swapchain.swapchain,
        u64::MAX,
        image_acquired_semaphore.semaphore,
        image_acquired_fence.fence,
        &mut image_index,
    ) {
        api::VK_SUCCESS => Ok(AcquireImageResults {
            image_index: Some(image_index as usize),
            need_new_swapchain: Some(swapchain.dimensions) != sdl_dimensions,
            image_acquired_semaphore: Some(image_acquired_semaphore),
            image_acquired_fence: Some(image_acquired_fence),
        }),
        api::VK_SUBOPTIMAL_KHR => Ok(AcquireImageResults {
            image_index: Some(image_index as usize),
            need_new_swapchain: true,
            image_acquired_semaphore: Some(image_acquired_semaphore),
            image_acquired_fence: Some(image_acquired_fence),
        }),
        api::VK_ERROR_OUT_OF_DATE_KHR => Ok(AcquireImageResults {
            image_index: None,
            need_new_swapchain: true,
            image_acquired_semaphore: None,
            image_acquired_fence: None,
        }),
        result => Err(VulkanError::VulkanError(result)),
    }?;
    if need_new_swapchain {
        vulkan_device.swapchain = match sdl_dimensions {
            Some(sdl_dimensions) => {
                let last_swapchain = vulkan_device.swapchain.take();
                Some(Arc::new(vulkan_device.create_swapchain_with_dimensions(
                    last_swapchain,
                    sdl_dimensions,
                )?))
            }
            None => None,
        };
    }
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
    if let Some(image_index) = image_index {
        let render_command_buffer = CommandBufferWrapper::new(
            device,
            vulkan_device
                .surface_state
                .as_ref()
                .unwrap()
                .render_queue_index,
            api::VK_COMMAND_BUFFER_LEVEL_PRIMARY,
        )?.begin(api::VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT, None)?;
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
        let framebuffer = &swapchain.framebuffers[image_index];
        device.vkCmdBeginRenderPass.unwrap()(
            render_command_buffer.command_buffer,
            &api::VkRenderPassBeginInfo {
                sType: api::VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO,
                pNext: null(),
                renderPass: graphics_pipeline.render_pass.render_pass,
                framebuffer: framebuffer.framebuffer,
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
        let render_pass_command_buffer_count = render_command_buffer_groups
            .iter()
            .map(
                |RenderCommandBufferGroup {
                     render_command_buffers,
                     ..
                 }| render_command_buffers.len(),
            ).sum();
        let mut render_pass_command_buffers = Vec::with_capacity(render_pass_command_buffer_count);
        for RenderCommandBufferGroup {
            render_command_buffers,
            final_transform,
        } in render_command_buffer_groups
        {
            let gl_to_vulkan_coordinates = math::Mat4::new(
                math::Vec4::new(1.0, 0.0, 0.0, 0.0),
                math::Vec4::new(0.0, -1.0, 0.0, 0.0),
                math::Vec4::new(0.0, 0.0, 0.5, 0.0),
                math::Vec4::new(0.0, 0.0, 0.5, 1.0),
            );
            for command_buffer in render_command_buffers.iter() {
                let generated_state = command_buffer.0.lock().unwrap().generate_state(
                    VulkanRenderCommandBufferGeneratedStateKey {
                        dimensions: swapchain.dimensions,
                        final_transform: gl_to_vulkan_coordinates * *final_transform,
                    },
                )?;
                for required_command_buffer in
                    &generated_state.referenced_objects.required_command_buffers
                {
                    required_command_buffer.assert_submitted();
                }
                render_pass_command_buffers.push(generated_state.command_buffer.command_buffer);
                generated_state.submit_tracker.set_submitted();
                referenced_objects.push(Box::new(generated_state));
            }
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
    }
    let render_completed_semaphore = if image_index.is_none() {
        None
    } else if vulkan_device.in_progress_present_semaphores.len()
        >= cmp::max(16, 2 * swapchain.framebuffers.len())
    {
        Some(
            vulkan_device
                .in_progress_present_semaphores
                .pop_front()
                .unwrap(),
        )
    } else {
        Some(SemaphoreWrapper::new(device.clone())?)
    };
    let fence = FenceWrapper::new(device.clone(), FenceState::Unsignaled)?;
    match device.vkQueueSubmit.unwrap()(
        vulkan_device.render_queue,
        1,
        &api::VkSubmitInfo {
            sType: api::VK_STRUCTURE_TYPE_SUBMIT_INFO,
            pNext: null(),
            waitSemaphoreCount: match &image_acquired_semaphore {
                Some(_) => 1,
                None => 0,
            },
            pWaitSemaphores: match &image_acquired_semaphore {
                Some(image_acquired_semaphore) => &image_acquired_semaphore.semaphore,
                None => null(),
            },
            pWaitDstStageMask: &api::VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT,
            commandBufferCount: command_buffers.len() as u32,
            pCommandBuffers: command_buffers.as_ptr(),
            signalSemaphoreCount: match &render_completed_semaphore {
                Some(_) => 1,
                None => 0,
            },
            pSignalSemaphores: match &render_completed_semaphore {
                Some(render_completed_semaphore) => &render_completed_semaphore.semaphore,
                None => null(),
            },
        },
        fence.fence,
    ) {
        api::VK_SUCCESS => {}
        result => return Err(VulkanError::VulkanError(result)),
    }
    if let Some(image_acquired_semaphore) = image_acquired_semaphore {
        referenced_objects.push(Box::new(image_acquired_semaphore));
    }
    match (
        image_index,
        image_acquired_fence,
        render_completed_semaphore,
    ) {
        (Some(image_index), Some(image_acquired_fence), Some(render_completed_semaphore)) => {
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
                    pResults: null_mut(),
                },
            ) {
                api::VK_SUCCESS => {}
                api::VK_SUBOPTIMAL_KHR | api::VK_ERROR_OUT_OF_DATE_KHR => {} // Handled on next acquire
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
                api::VK_SUCCESS => {}
                result => return Err(VulkanError::VulkanError(result)),
            }
        }
        (None, None, None) => {}
        _ => unreachable!(),
    }
    referenced_objects.push(Box::new(swapchain));
    vulkan_device
        .in_progress_operations
        .push_back((fence, referenced_objects));
    Ok(())
}
