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
    api, create_fence, create_initialized_device_buffer, create_initialized_device_image_set,
    create_signaled_fence, get_fence_vulkan_state, get_fence_wait_completed,
    get_vulkan_device_image_set_implementation, get_vulkan_staging_image_set_buffer, null_or_zero,
    set_push_constants, set_push_constants_initial_transform, BufferWrapper, DescriptorSetWrapper,
    DeviceWrapper, FenceState, FenceWrapper, GraphicsPipelineWrapper, ImageViewWrapper,
    PipelineLayoutWrapper, PushConstants, RenderPassWrapper, Result, SamplerWrapper,
    SemaphoreWrapper, VulkanBuffer, VulkanDevice, VulkanDeviceBuffer, VulkanDeviceImageSet,
    VulkanDeviceImageSetImplementation, VulkanError, VulkanFence, VulkanStagingArrayGetSharedState,
    VulkanStagingBuffer, VulkanStagingImageSet, COLOR_ATTACHEMENT_INDEX, DEPTH_ATTACHEMENT_INDEX,
    SAMPLERS_DESCRIPTOR_SET_INDEX,
};
use math;
use renderer::{
    CommandBuffer, Device, GenericArray, IndexBufferElement, LoaderCommandBufferBuilder,
    RenderCommandBufferBuilder, RenderCommandBufferGroup, Slice, VertexBufferElement,
};
use sdl;
use std::cmp;
use std::mem;
use std::ptr::{null, null_mut};
use std::sync::atomic::*;
use std::sync::{Arc, Mutex};
use std::u64;
use voxels_image::{Image, Pixel};

pub struct CommandPoolWrapper {
    pub device: Arc<DeviceWrapper>,
    pub command_pool: api::VkCommandPool,
}

unsafe impl Send for CommandPoolWrapper {}
unsafe impl Sync for CommandPoolWrapper {}

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
unsafe impl Sync for CommandBufferWrapper {}

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

pub trait CommandBufferSubmitTracker: Sync + Send + Clone + 'static + Default {
    fn assert_submitted(&self);
    fn set_submitted(&self);
}

#[derive(Clone)]
pub struct ActiveCommandBufferSubmitTracker {
    submitted_flag: Arc<AtomicBool>,
}

impl Default for ActiveCommandBufferSubmitTracker {
    fn default() -> Self {
        Self {
            submitted_flag: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl CommandBufferSubmitTracker for ActiveCommandBufferSubmitTracker {
    fn assert_submitted(&self) {
        assert!(self.submitted_flag.load(Ordering::Acquire));
    }
    fn set_submitted(&self) {
        self.submitted_flag.store(true, Ordering::Release);
    }
}

#[derive(Clone, Default)]
pub struct InactiveCommandBufferSubmitTracker;

impl CommandBufferSubmitTracker for InactiveCommandBufferSubmitTracker {
    fn assert_submitted(&self) {}
    fn set_submitted(&self) {
        unreachable!();
    }
}

pub struct CommandBufferReferencedObjects {
    required_command_buffers: Vec<ActiveCommandBufferSubmitTracker>,
    shared_buffers: Vec<Arc<BufferWrapper>>,
    shared_image_view_vecs: Vec<Arc<Vec<ImageViewWrapper>>>,
    shared_sampler_vecs: Vec<Arc<Vec<SamplerWrapper>>>,
    shared_descriptor_sets: Vec<Arc<DescriptorSetWrapper>>,
    staging_array_fence_wait_completed_list: Vec<Arc<Mutex<Option<Arc<AtomicBool>>>>>,
}

impl Default for CommandBufferReferencedObjects {
    fn default() -> Self {
        Self {
            required_command_buffers: Vec::new(),
            shared_buffers: Vec::new(),
            shared_image_view_vecs: Vec::new(),
            shared_sampler_vecs: Vec::new(),
            shared_descriptor_sets: Vec::new(),
            staging_array_fence_wait_completed_list: Vec::new(),
        }
    }
}

pub struct VulkanLoaderCommandBuffer {
    command_buffer: CommandBufferWrapper,
    referenced_objects: CommandBufferReferencedObjects,
    submit_tracker: ActiveCommandBufferSubmitTracker,
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
                referenced_objects: Default::default(),
                submit_tracker: Default::default(),
            },
        })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum CopyKind {
    Initial,
    Normal,
}

unsafe fn copy_buffer_to_device<T: Copy + Send + Sync + 'static>(
    builder: &mut VulkanLoaderCommandBufferBuilder,
    staging_buffer: Slice<T, &VulkanStagingBuffer<T>>,
    device_buffer: Slice<T, &VulkanDeviceBuffer<T, ActiveCommandBufferSubmitTracker>>,
    copy_kind: CopyKind,
) -> Result<()> {
    assert_eq!(staging_buffer.len(), device_buffer.len());
    let command_buffer = &builder.0.command_buffer;
    let device = &command_buffer.command_pool.device;
    if CopyKind::Initial != copy_kind {
        builder
            .0
            .referenced_objects
            .required_command_buffers
            .push(device_buffer.underlying().submit_tracker());
        device.vkCmdPipelineBarrier.unwrap()(
            command_buffer.command_buffer,
            api::VK_PIPELINE_STAGE_VERTEX_INPUT_BIT,
            api::VK_PIPELINE_STAGE_TRANSFER_BIT,
            0,
            0,
            null(),
            1,
            &api::VkBufferMemoryBarrier {
                sType: api::VK_STRUCTURE_TYPE_BUFFER_MEMORY_BARRIER,
                pNext: null(),
                srcAccessMask: api::VK_ACCESS_VERTEX_ATTRIBUTE_READ_BIT,
                dstAccessMask: api::VK_ACCESS_TRANSFER_WRITE_BIT,
                srcQueueFamilyIndex: command_buffer.queue_family_index,
                dstQueueFamilyIndex: command_buffer.queue_family_index,
                buffer: device_buffer.underlying().buffer().buffer,
                offset: (device_buffer.start() * mem::size_of::<T>()) as u64,
                size: (device_buffer.len() * mem::size_of::<T>()) as u64,
            },
            0,
            null(),
        );
    }
    device.vkCmdCopyBuffer.unwrap()(
        command_buffer.command_buffer,
        staging_buffer.underlying().buffer().buffer,
        device_buffer.underlying().buffer().buffer,
        1,
        &api::VkBufferCopy {
            srcOffset: (staging_buffer.start() * mem::size_of::<T>()) as u64,
            dstOffset: (device_buffer.start() * mem::size_of::<T>()) as u64,
            size: (device_buffer.len() * mem::size_of::<T>()) as u64,
        },
    );
    builder
        .0
        .referenced_objects
        .shared_buffers
        .push(staging_buffer.underlying().buffer().clone());
    builder
        .0
        .referenced_objects
        .staging_array_fence_wait_completed_list
        .push(
            staging_buffer
                .underlying()
                .shared_state()
                .device_access_fence_wait_completed
                .clone(),
        );
    builder
        .0
        .referenced_objects
        .shared_buffers
        .push(device_buffer.underlying().buffer().clone());
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
            srcQueueFamilyIndex: command_buffer.queue_family_index,
            dstQueueFamilyIndex: command_buffer.queue_family_index,
            buffer: device_buffer.underlying().buffer().buffer,
            offset: (device_buffer.start() * mem::size_of::<T>()) as u64,
            size: (device_buffer.len() * mem::size_of::<T>()) as u64,
        },
        0,
        null(),
    );
    Ok(())
}

unsafe fn copy_image_set_to_device(
    builder: &mut VulkanLoaderCommandBufferBuilder,
    staging_image_set_slice: Slice<Image, &VulkanStagingImageSet>,
    device_image_set_slice: Slice<Image, &VulkanDeviceImageSet<ActiveCommandBufferSubmitTracker>>,
    copy_kind: CopyKind,
) -> Result<()> {
    let command_buffer = &builder.0.command_buffer;
    let device = &command_buffer.command_pool.device;
    let staging_buffer = get_vulkan_staging_image_set_buffer(staging_image_set_slice.underlying());
    let device_image_set =
        get_vulkan_device_image_set_implementation(device_image_set_slice.underlying());
    if CopyKind::Initial != copy_kind {
        builder
            .0
            .referenced_objects
            .required_command_buffers
            .push(device_image_set.submit_tracker.clone());
    }
    let mut start_image_memory_barriers = Vec::new();
    let mut end_image_memory_barriers = Vec::new();
    let start_source_access_mask = match copy_kind {
        CopyKind::Initial => 0,
        CopyKind::Normal => api::VK_ACCESS_SHADER_READ_BIT,
    };
    let start_old_layout = match copy_kind {
        CopyKind::Initial => api::VK_IMAGE_LAYOUT_UNDEFINED,
        CopyKind::Normal => api::VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL,
    };
    for image in &*device_image_set.images {
        start_image_memory_barriers.push(api::VkImageMemoryBarrier {
            sType: api::VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER,
            pNext: null(),
            srcAccessMask: start_source_access_mask,
            dstAccessMask: api::VK_ACCESS_TRANSFER_WRITE_BIT,
            oldLayout: start_old_layout,
            newLayout: api::VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
            srcQueueFamilyIndex: command_buffer.queue_family_index,
            dstQueueFamilyIndex: command_buffer.queue_family_index,
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
            srcQueueFamilyIndex: command_buffer.queue_family_index,
            dstQueueFamilyIndex: command_buffer.queue_family_index,
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
        match copy_kind {
            CopyKind::Initial => api::VK_PIPELINE_STAGE_HOST_BIT,
            CopyKind::Normal => {
                api::VK_PIPELINE_STAGE_HOST_BIT | api::VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT
            }
        },
        api::VK_PIPELINE_STAGE_TRANSFER_BIT,
        0,
        0,
        null(),
        0,
        null(),
        start_image_memory_barriers.len() as u32,
        start_image_memory_barriers.as_ptr(),
    );
    builder
        .0
        .referenced_objects
        .shared_image_view_vecs
        .push(device_image_set.images.clone());
    let width = device_image_set.dimensions.x as usize;
    let height = device_image_set.dimensions.y as usize;
    let image_size = width * height * mem::size_of::<Pixel>();
    let mut buffer_image_copy_structs = Vec::new();
    for (image_index, image) in device_image_set.images.iter().enumerate() {
        if image_index as u32 >= device_image_set.valid_image_count {
            if CopyKind::Initial == copy_kind {
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
            }
        } else {
            buffer_image_copy_structs.clear();
            let image_layer_count =
                if image_index + 1 == device_image_set.valid_image_count as usize {
                    device_image_set.last_image_layer_count
                } else {
                    device_image_set.image_layer_count
                };
            for layer in 0..image_layer_count {
                let image_layer_index =
                    image_index * device_image_set.image_layer_count as usize + layer as usize;
                if image_layer_index < device_image_set_slice.start() {
                    continue;
                }
                if image_layer_index - device_image_set_slice.start()
                    >= device_image_set_slice.len()
                {
                    continue;
                }
                buffer_image_copy_structs.push(api::VkBufferImageCopy {
                    bufferOffset: (image_size
                        * (image_layer_index + staging_image_set_slice.start()))
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
            if !buffer_image_copy_structs.is_empty() {
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
    }
    builder
        .0
        .referenced_objects
        .shared_buffers
        .push(staging_buffer.clone());
    builder
        .0
        .referenced_objects
        .staging_array_fence_wait_completed_list
        .push(
            staging_image_set_slice
                .underlying()
                .shared_state()
                .device_access_fence_wait_completed
                .clone(),
        );
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
    Ok(())
}

impl LoaderCommandBufferBuilder for VulkanLoaderCommandBufferBuilder {
    type Error = VulkanError;
    type CommandBuffer = VulkanLoaderCommandBuffer;
    type StagingVertexBuffer = VulkanStagingBuffer<VertexBufferElement>;
    type UninitializedDeviceVertexBuffer =
        VulkanDeviceBuffer<VertexBufferElement, InactiveCommandBufferSubmitTracker>;
    type DeviceVertexBuffer =
        VulkanDeviceBuffer<VertexBufferElement, ActiveCommandBufferSubmitTracker>;
    type StagingIndexBuffer = VulkanStagingBuffer<IndexBufferElement>;
    type UninitializedDeviceIndexBuffer =
        VulkanDeviceBuffer<IndexBufferElement, InactiveCommandBufferSubmitTracker>;
    type DeviceIndexBuffer =
        VulkanDeviceBuffer<IndexBufferElement, ActiveCommandBufferSubmitTracker>;
    type StagingImageSet = VulkanStagingImageSet;
    type UninitializedDeviceImageSet = VulkanDeviceImageSet<InactiveCommandBufferSubmitTracker>;
    type DeviceImageSet = VulkanDeviceImageSet<ActiveCommandBufferSubmitTracker>;
    fn initialize_vertex_buffer(
        &mut self,
        staging_buffer: Slice<VertexBufferElement, &VulkanStagingBuffer<VertexBufferElement>>,
        device_buffer: VulkanDeviceBuffer<VertexBufferElement, InactiveCommandBufferSubmitTracker>,
    ) -> Result<VulkanDeviceBuffer<VertexBufferElement, ActiveCommandBufferSubmitTracker>> {
        assert_eq!(staging_buffer.len(), device_buffer.len());
        let device_buffer =
            create_initialized_device_buffer(device_buffer, self.0.submit_tracker.clone());
        unsafe {
            copy_buffer_to_device(
                self,
                staging_buffer,
                device_buffer.slice_ref(..),
                CopyKind::Initial,
            )?;
        }
        Ok(device_buffer)
    }
    fn initialize_index_buffer(
        &mut self,
        staging_buffer: Slice<IndexBufferElement, &VulkanStagingBuffer<IndexBufferElement>>,
        device_buffer: VulkanDeviceBuffer<IndexBufferElement, InactiveCommandBufferSubmitTracker>,
    ) -> Result<VulkanDeviceBuffer<IndexBufferElement, ActiveCommandBufferSubmitTracker>> {
        assert_eq!(staging_buffer.len(), device_buffer.len());
        let device_buffer =
            create_initialized_device_buffer(device_buffer, self.0.submit_tracker.clone());
        unsafe {
            copy_buffer_to_device(
                self,
                staging_buffer,
                device_buffer.slice_ref(..),
                CopyKind::Initial,
            )?;
        }
        Ok(device_buffer)
    }
    fn initialize_image_set(
        &mut self,
        staging_image_set: Slice<Image, &VulkanStagingImageSet>,
        device_image_set: VulkanDeviceImageSet<InactiveCommandBufferSubmitTracker>,
    ) -> Result<VulkanDeviceImageSet<ActiveCommandBufferSubmitTracker>> {
        assert_eq!(staging_image_set.len(), device_image_set.len());
        let device_image_set =
            create_initialized_device_image_set(device_image_set, self.0.submit_tracker.clone());
        unsafe {
            copy_image_set_to_device(
                self,
                staging_image_set,
                device_image_set.slice_ref(..),
                CopyKind::Initial,
            )?;
        }
        Ok(device_image_set)
    }
    fn copy_vertex_buffer_to_device(
        &mut self,
        staging_vertex_buffer: Slice<
            VertexBufferElement,
            &VulkanStagingBuffer<VertexBufferElement>,
        >,
        device_vertex_buffer: Slice<
            VertexBufferElement,
            &VulkanDeviceBuffer<VertexBufferElement, ActiveCommandBufferSubmitTracker>,
        >,
    ) -> Result<()> {
        unsafe {
            copy_buffer_to_device(
                self,
                staging_vertex_buffer,
                device_vertex_buffer,
                CopyKind::Normal,
            )
        }
    }
    fn copy_index_buffer_to_device(
        &mut self,
        staging_index_buffer: Slice<IndexBufferElement, &VulkanStagingBuffer<IndexBufferElement>>,
        device_index_buffer: Slice<
            IndexBufferElement,
            &VulkanDeviceBuffer<IndexBufferElement, ActiveCommandBufferSubmitTracker>,
        >,
    ) -> Result<()> {
        unsafe {
            copy_buffer_to_device(
                self,
                staging_index_buffer,
                device_index_buffer,
                CopyKind::Normal,
            )
        }
    }
    fn copy_image_set_to_device(
        &mut self,
        staging_image_set_slice: Slice<Image, &VulkanStagingImageSet>,
        device_image_set_slice: Slice<
            Image,
            &VulkanDeviceImageSet<ActiveCommandBufferSubmitTracker>,
        >,
    ) -> Result<()> {
        unsafe {
            copy_image_set_to_device(
                self,
                staging_image_set_slice,
                device_image_set_slice,
                CopyKind::Normal,
            )
        }
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
        image_set: VulkanDeviceImageSetImplementation<ActiveCommandBufferSubmitTracker>,
    },
    SetInitialTransform {
        transform: math::Mat4<f32>,
    },
    Draw {
        vertex_buffer: Arc<BufferWrapper>,
        vertex_buffer_submit_tracker: ActiveCommandBufferSubmitTracker,
        index_buffer: Arc<BufferWrapper>,
        index_buffer_submit_tracker: ActiveCommandBufferSubmitTracker,
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
        use math::Mappable;
        let into_compare_key = |v: &Self| (v.dimensions, v.final_transform.map(|v| v.to_bits()));
        into_compare_key(self) == into_compare_key(rhs)
    }
}

struct VulkanRenderCommandBufferGeneratedState {
    key: VulkanRenderCommandBufferGeneratedStateKey,
    command_buffer: CommandBufferWrapper,
    referenced_objects: CommandBufferReferencedObjects,
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
        let mut referenced_objects: CommandBufferReferencedObjects = Default::default();
        for render_command in &self.render_commands {
            match render_command.clone() {
                RenderCommand::SetImageSet {
                    image_set:
                        VulkanDeviceImageSetImplementation {
                            images,
                            dimensions: _,
                            total_layer_count: _,
                            image_layer_count: _,
                            last_image_layer_count: _,
                            valid_image_count: _,
                            samplers,
                            descriptor_set,
                            submit_tracker,
                        },
                } => {
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
                    referenced_objects.shared_sampler_vecs.push(samplers);
                    referenced_objects
                        .shared_descriptor_sets
                        .push(descriptor_set);
                    referenced_objects
                        .required_command_buffers
                        .push(submit_tracker);
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
                    vertex_buffer,
                    vertex_buffer_submit_tracker,
                    index_buffer,
                    index_buffer_submit_tracker,
                    index_count,
                    first_index,
                    vertex_offset,
                } => {
                    self.device.vkCmdBindVertexBuffers.unwrap()(
                        command_buffer.command_buffer,
                        0,
                        1,
                        &vertex_buffer.buffer,
                        &0,
                    );
                    referenced_objects.shared_buffers.push(vertex_buffer);
                    referenced_objects
                        .required_command_buffers
                        .push(vertex_buffer_submit_tracker);
                    self.device.vkCmdBindIndexBuffer.unwrap()(
                        command_buffer.command_buffer,
                        index_buffer.buffer,
                        0,
                        api::VK_INDEX_TYPE_UINT16,
                    );
                    referenced_objects.shared_buffers.push(index_buffer);
                    referenced_objects
                        .required_command_buffers
                        .push(index_buffer_submit_tracker);
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
        });
        self.generated_state = Some(retval.clone());
        Ok(retval)
    }
}

#[derive(Clone)]
pub struct VulkanRenderCommandBuffer(Arc<Mutex<VulkanRenderCommandBufferState>>);

impl CommandBuffer for VulkanRenderCommandBuffer {}

pub struct VulkanRenderCommandBufferBuilder {
    render_commands: Vec<RenderCommand>,
    device: Arc<DeviceWrapper>,
    queue_family_index: u32,
    render_pass: Arc<RenderPassWrapper>,
    pipeline_layout: Arc<PipelineLayoutWrapper>,
    graphics_pipeline: Arc<GraphicsPipelineWrapper>,
    did_set_initial_transform: bool,
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
            did_set_image_set: false,
        }
    }
}

impl RenderCommandBufferBuilder for VulkanRenderCommandBufferBuilder {
    type Error = VulkanError;
    type CommandBuffer = VulkanRenderCommandBuffer;
    type DeviceVertexBuffer =
        VulkanDeviceBuffer<VertexBufferElement, ActiveCommandBufferSubmitTracker>;
    type DeviceIndexBuffer =
        VulkanDeviceBuffer<IndexBufferElement, ActiveCommandBufferSubmitTracker>;
    type DeviceImageSet = VulkanDeviceImageSet<ActiveCommandBufferSubmitTracker>;
    fn set_image_set(
        &mut self,
        image_set: &VulkanDeviceImageSet<ActiveCommandBufferSubmitTracker>,
    ) {
        self.render_commands.push(RenderCommand::SetImageSet {
            image_set: get_vulkan_device_image_set_implementation(image_set).clone(),
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
    fn draw(
        &mut self,
        vertex_buffer: Slice<
            VertexBufferElement,
            &VulkanDeviceBuffer<VertexBufferElement, ActiveCommandBufferSubmitTracker>,
        >,
        index_buffer: Slice<
            IndexBufferElement,
            &VulkanDeviceBuffer<IndexBufferElement, ActiveCommandBufferSubmitTracker>,
        >,
    ) {
        assert!(
            index_buffer.len() % 3 == 0,
            "must be whole number of triangles"
        );
        assert!(self.did_set_image_set);
        if index_buffer.len() > 0 {
            if !self.did_set_initial_transform {
                self.set_initial_transform(math::Mat4::identity());
            }
            self.render_commands.push(RenderCommand::Draw {
                vertex_buffer: vertex_buffer.underlying().buffer().clone(),
                vertex_buffer_submit_tracker: vertex_buffer.underlying().submit_tracker(),
                index_buffer: index_buffer.underlying().buffer().clone(),
                index_buffer_submit_tracker: vertex_buffer.underlying().submit_tracker(),
                index_count: index_buffer.len() as u32,
                first_index: index_buffer.start() as u32,
                vertex_offset: vertex_buffer.start() as u32,
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
) -> Result<VulkanFence> {
    vulkan_device.free_finished_objects()?;
    if loader_command_buffers.is_empty() {
        return Ok(create_signaled_fence());
    }
    let device = &vulkan_device.device_reference.device;
    let mut command_buffers = Vec::with_capacity(loader_command_buffers.len());
    let fence = create_fence(device.clone())?;
    {
        let mut locked_fence_state = get_fence_vulkan_state(&fence).lock().unwrap();
        let locked_fence_state = (*locked_fence_state).as_mut().unwrap();
        let referenced_objects = &mut locked_fence_state.referenced_objects;
        for command_buffer in loader_command_buffers.drain(..) {
            command_buffers.push(command_buffer.command_buffer.command_buffer);
            command_buffer.submit_tracker.set_submitted();
            for required_command_buffer in command_buffer
                .referenced_objects
                .required_command_buffers
                .iter()
            {
                required_command_buffer.assert_submitted();
            }
            for staging_array_fence_wait_completed in command_buffer
                .referenced_objects
                .staging_array_fence_wait_completed_list
                .iter()
            {
                *staging_array_fence_wait_completed.lock().unwrap() =
                    Some(get_fence_wait_completed(&fence).clone());
            }
            referenced_objects.push(Box::new(command_buffer));
        }
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
                locked_fence_state.fence.fence,
            )
        } {
            api::VK_SUCCESS => {}
            result => return Err(VulkanError::VulkanError(result)),
        }
    }
    vulkan_device
        .in_progress_operations
        .push_back(fence.clone());
    Ok(fence)
}

unsafe fn debug_device_wait_idle(device: &DeviceWrapper) {
    if true {
        device.vkDeviceWaitIdle.unwrap()(device.device);
    }
}

pub unsafe fn render_frame(
    vulkan_device: &mut VulkanDevice,
    clear_color: math::Vec4<f32>,
    loader_command_buffers: &mut Vec<VulkanLoaderCommandBuffer>,
    render_command_buffer_groups: &[RenderCommandBufferGroup<VulkanRenderCommandBuffer>],
) -> Result<VulkanFence> {
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
    let fence = create_fence(device.clone())?;
    {
        let mut locked_fence_state = get_fence_vulkan_state(&fence).lock().unwrap();
        let locked_fence_state = (*locked_fence_state).as_mut().unwrap();
        let mut command_buffers = Vec::with_capacity(loader_command_buffers.len() + 1);
        let referenced_objects = &mut locked_fence_state.referenced_objects;
        for command_buffer in loader_command_buffers.drain(..) {
            command_buffers.push(command_buffer.command_buffer.command_buffer);
            command_buffer.submit_tracker.set_submitted();
            for required_command_buffer in command_buffer
                .referenced_objects
                .required_command_buffers
                .iter()
            {
                required_command_buffer.assert_submitted();
            }
            for staging_array_fence_wait_completed in command_buffer
                .referenced_objects
                .staging_array_fence_wait_completed_list
                .iter()
            {
                *staging_array_fence_wait_completed.lock().unwrap() =
                    Some(get_fence_wait_completed(&fence).clone());
            }
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
            let mut render_pass_command_buffers =
                Vec::with_capacity(render_pass_command_buffer_count);
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
                    for required_command_buffer in generated_state
                        .referenced_objects
                        .required_command_buffers
                        .iter()
                    {
                        required_command_buffer.assert_submitted();
                    }
                    for staging_array_fence_wait_completed in generated_state
                        .referenced_objects
                        .staging_array_fence_wait_completed_list
                        .iter()
                    {
                        *staging_array_fence_wait_completed.lock().unwrap() =
                            Some(get_fence_wait_completed(&fence).clone());
                    }
                    render_pass_command_buffers.push(generated_state.command_buffer.command_buffer);
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
            locked_fence_state.fence.fence,
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
                debug_device_wait_idle(device);
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
    }
    vulkan_device
        .in_progress_operations
        .push_back(fence.clone());
    Ok(fence)
}
