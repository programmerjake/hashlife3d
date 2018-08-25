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
    api, null_or_zero, transmute_from_byte_slice, ActiveCommandBufferSubmitTracker,
    CommandBufferSubmitTracker, DeviceMemoryPoolAllocation, DeviceMemoryPools, DeviceWrapper,
    InactiveCommandBufferSubmitTracker, Result, VulkanError,
};
use renderer::{
    Buffer, DeviceBuffer, DeviceGenericArray, GenericArray, IndexBufferElement, StagingBuffer,
    StagingGenericArray, UninitializedDeviceBuffer, UninitializedDeviceGenericArray,
    VertexBufferElement,
};
use std::cmp;
use std::convert;
use std::marker::PhantomData;
use std::mem;
use std::ptr::{null, NonNull};
use std::sync::Arc;

pub struct BufferWrapper {
    pub device: Arc<DeviceWrapper>,
    pub buffer: api::VkBuffer,
    pub device_memory: Option<DeviceMemoryPoolAllocation>,
}

impl Drop for BufferWrapper {
    fn drop(&mut self) {
        unsafe {
            self.device.vkDestroyBuffer.unwrap()(self.device.device, self.buffer, null());
        }
    }
}

impl BufferWrapper {
    pub unsafe fn new(
        device: Arc<DeviceWrapper>,
        size: api::VkDeviceSize,
        usage: api::VkBufferUsageFlags,
        sharing_mode: api::VkSharingMode,
        queue_family_indices: &[u32],
    ) -> Result<BufferWrapper> {
        let mut buffer = null_or_zero();
        match device.vkCreateBuffer.unwrap()(
            device.device,
            &api::VkBufferCreateInfo {
                sType: api::VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO,
                pNext: null(),
                flags: 0,
                size: size,
                usage: usage,
                sharingMode: sharing_mode,
                queueFamilyIndexCount: queue_family_indices.len() as u32,
                pQueueFamilyIndices: queue_family_indices.as_ptr(),
            },
            null(),
            &mut buffer,
        ) {
            api::VK_SUCCESS => Ok(BufferWrapper {
                device: device,
                buffer: buffer,
                device_memory: None,
            }),
            result => Err(VulkanError::VulkanError(result)),
        }
    }
    pub unsafe fn allocate_and_bind_memory(
        mut self,
        device_memory_pools: &DeviceMemoryPools,
        element_alignment: usize,
        preferred_properties: Option<api::VkMemoryPropertyFlags>,
        required_properties: api::VkMemoryPropertyFlags,
    ) -> Result<Self> {
        assert!(self.device_memory.is_none());
        let mut memory_requirements = mem::zeroed();
        self.device.vkGetBufferMemoryRequirements.unwrap()(
            self.device.device,
            self.buffer,
            &mut memory_requirements,
        );
        memory_requirements.alignment =
            cmp::max(memory_requirements.alignment, element_alignment as u64);
        let memory_allocation = device_memory_pools.allocate_from_memory_requirements(
            memory_requirements,
            preferred_properties,
            required_properties,
        )?;
        match self.device.vkBindBufferMemory.unwrap()(
            self.device.device,
            self.buffer,
            memory_allocation.get_device_memory().get_device_memory(),
            memory_allocation.get_offset(),
        ) {
            api::VK_SUCCESS => {
                self.device_memory = Some(memory_allocation);
                Ok(self)
            }
            result => Err(VulkanError::VulkanError(result)),
        }
    }
}

unsafe impl Send for BufferWrapper {}
unsafe impl Sync for BufferWrapper {}

pub trait VulkanBuffer<T: Copy + Sync + Send + 'static>: Buffer<T> {
    type SubmitTracker: CommandBufferSubmitTracker;
    fn buffer(&self) -> &Arc<BufferWrapper>;
    fn submit_tracker(&self) -> Self::SubmitTracker;
}

pub struct VulkanStagingBuffer<T: Copy + Sync + Send + 'static> {
    buffer: Arc<BufferWrapper>,
    mapped_memory: NonNull<[T]>,
}

unsafe impl<T: Copy + Sync + Send + 'static> Sync for VulkanStagingBuffer<T> {}

unsafe impl<T: Copy + Sync + Send + 'static> Send for VulkanStagingBuffer<T> {}

impl<T: Copy + Sync + Send + 'static> GenericArray<T> for VulkanStagingBuffer<T> {
    fn len(&self) -> usize {
        unsafe { self.mapped_memory.as_ref() }.len()
    }
}

impl<T: Copy + Sync + Send + 'static> convert::AsRef<[T]> for VulkanStagingBuffer<T> {
    fn as_ref(&self) -> &[T] {
        unsafe { self.mapped_memory.as_ref() }
    }
}

impl<T: Copy + Sync + Send + 'static> convert::AsMut<[T]> for VulkanStagingBuffer<T> {
    fn as_mut(&mut self) -> &mut [T] {
        unsafe { self.mapped_memory.as_mut() }
    }
}

impl<T: Copy + Sync + Send + 'static> StagingGenericArray<T> for VulkanStagingBuffer<T> {}

impl<T: Copy + Sync + Send + 'static> Buffer<T> for VulkanStagingBuffer<T> {}

impl<T: Copy + Sync + Send + 'static> StagingBuffer<T> for VulkanStagingBuffer<T> {}

impl<T: Copy + Sync + Send + 'static> VulkanBuffer<T> for VulkanStagingBuffer<T> {
    type SubmitTracker = InactiveCommandBufferSubmitTracker;
    fn buffer(&self) -> &Arc<BufferWrapper> {
        &self.buffer
    }
    fn submit_tracker(&self) -> InactiveCommandBufferSubmitTracker {
        InactiveCommandBufferSubmitTracker
    }
}

pub struct VulkanDeviceBuffer<T: Copy + Sync + Send + 'static, CBST: CommandBufferSubmitTracker> {
    buffer: Arc<BufferWrapper>,
    len: usize,
    _phantom: PhantomData<&'static T>,
    submit_tracker: CBST,
}

impl<T: Copy + Sync + Send + 'static, CBST: CommandBufferSubmitTracker> GenericArray<T>
    for VulkanDeviceBuffer<T, CBST>
{
    fn len(&self) -> usize {
        self.len
    }
}

impl<T: Copy + Sync + Send + 'static> DeviceGenericArray<T>
    for VulkanDeviceBuffer<T, ActiveCommandBufferSubmitTracker>
{}

impl<T: Copy + Sync + Send + 'static> UninitializedDeviceGenericArray<T>
    for VulkanDeviceBuffer<T, InactiveCommandBufferSubmitTracker>
{}

impl<T: Copy + Sync + Send + 'static, CBST: CommandBufferSubmitTracker> Buffer<T>
    for VulkanDeviceBuffer<T, CBST>
{}

impl<T: Copy + Sync + Send + 'static> DeviceBuffer<T>
    for VulkanDeviceBuffer<T, ActiveCommandBufferSubmitTracker>
{}

impl<T: Copy + Sync + Send + 'static> UninitializedDeviceBuffer<T>
    for VulkanDeviceBuffer<T, InactiveCommandBufferSubmitTracker>
{}

impl<T: Copy + Sync + Send + 'static, CBST: CommandBufferSubmitTracker> VulkanBuffer<T>
    for VulkanDeviceBuffer<T, CBST>
{
    type SubmitTracker = CBST;
    fn buffer(&self) -> &Arc<BufferWrapper> {
        &self.buffer
    }
    fn submit_tracker(&self) -> CBST {
        self.submit_tracker.clone()
    }
}

pub fn create_initialized_device_buffer<T: Copy + Sync + Send + 'static>(
    device_buffer: VulkanDeviceBuffer<T, InactiveCommandBufferSubmitTracker>,
    submit_tracker: ActiveCommandBufferSubmitTracker,
) -> VulkanDeviceBuffer<T, ActiveCommandBufferSubmitTracker> {
    VulkanDeviceBuffer {
        buffer: device_buffer.buffer,
        len: device_buffer.len,
        _phantom: PhantomData,
        submit_tracker: submit_tracker,
    }
}

const STAGING_BUFFER_USAGE_FLAGS: api::VkBufferUsageFlags = api::VK_BUFFER_USAGE_TRANSFER_SRC_BIT;

pub unsafe fn create_staging_buffer<T: Copy + Sync + Send + 'static>(
    device: Arc<DeviceWrapper>,
    device_memory_pools: &DeviceMemoryPools,
    element_count: usize,
) -> Result<VulkanStagingBuffer<T>> {
    let buffer = BufferWrapper::new(
        device.clone(),
        cmp::max(1, element_count) as u64 * mem::size_of::<T>() as u64,
        STAGING_BUFFER_USAGE_FLAGS,
        api::VK_SHARING_MODE_EXCLUSIVE,
        &[],
    )?;
    let buffer = buffer.allocate_and_bind_memory(
        device_memory_pools,
        mem::align_of::<T>(),
        None,
        api::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | api::VK_MEMORY_PROPERTY_HOST_COHERENT_BIT,
    )?;
    let mut mapped_memory = buffer
        .device_memory
        .as_ref()
        .unwrap()
        .get_mapped_memory()
        .unwrap();
    let mapped_memory = transmute_from_byte_slice(
        (&mut mapped_memory.as_mut()[..(element_count as usize * mem::size_of::<T>())]).into(),
    );
    assert_eq!(mapped_memory.as_ref().len(), element_count);
    Ok(VulkanStagingBuffer {
        buffer: Arc::new(buffer),
        mapped_memory: mapped_memory,
    })
}

const DEVICE_VERTEX_BUFFER_USAGE_FLAGS: api::VkBufferUsageFlags =
    api::VK_BUFFER_USAGE_TRANSFER_DST_BIT | api::VK_BUFFER_USAGE_VERTEX_BUFFER_BIT;

pub unsafe fn create_device_vertex_buffer(
    device: Arc<DeviceWrapper>,
    device_memory_pools: &DeviceMemoryPools,
    element_count: usize,
) -> Result<VulkanDeviceBuffer<VertexBufferElement, InactiveCommandBufferSubmitTracker>> {
    let buffer = BufferWrapper::new(
        device,
        cmp::max(1, element_count) as u64 * mem::size_of::<VertexBufferElement>() as u64,
        DEVICE_VERTEX_BUFFER_USAGE_FLAGS,
        api::VK_SHARING_MODE_EXCLUSIVE,
        &[],
    )?;
    let buffer = buffer.allocate_and_bind_memory(
        device_memory_pools,
        1,
        None,
        api::VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
    )?;
    Ok(VulkanDeviceBuffer {
        buffer: Arc::new(buffer),
        len: element_count,
        _phantom: PhantomData,
        submit_tracker: InactiveCommandBufferSubmitTracker,
    })
}

const DEVICE_INDEX_BUFFER_USAGE_FLAGS: api::VkBufferUsageFlags =
    api::VK_BUFFER_USAGE_TRANSFER_DST_BIT | api::VK_BUFFER_USAGE_INDEX_BUFFER_BIT;

pub unsafe fn create_device_index_buffer(
    device: Arc<DeviceWrapper>,
    device_memory_pools: &DeviceMemoryPools,
    element_count: usize,
) -> Result<VulkanDeviceBuffer<IndexBufferElement, InactiveCommandBufferSubmitTracker>> {
    let buffer = BufferWrapper::new(
        device,
        cmp::max(1, element_count) as u64 * mem::size_of::<IndexBufferElement>() as u64,
        DEVICE_INDEX_BUFFER_USAGE_FLAGS,
        api::VK_SHARING_MODE_EXCLUSIVE,
        &[],
    )?;
    let buffer = buffer.allocate_and_bind_memory(
        device_memory_pools,
        1,
        None,
        api::VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
    )?;
    Ok(VulkanDeviceBuffer {
        buffer: Arc::new(buffer),
        len: element_count,
        _phantom: PhantomData,
        submit_tracker: InactiveCommandBufferSubmitTracker,
    })
}
