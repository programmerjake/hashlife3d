use super::{
    api, null_or_zero, CommandBufferSubmitTracker, DeviceMemoryPoolAllocation, DeviceMemoryPools,
    DeviceWrapper, Result, SemaphoreWrapper, VulkanError,
};
use renderer::{
    DeviceIndexBuffer, DeviceVertexBuffer, IndexBufferElement, StagingIndexBuffer,
    StagingVertexBuffer, VertexBufferElement,
};
use std::cmp;
use std::mem;
use std::ptr::null;
use std::slice;
use std::sync::{Arc, Mutex};

pub struct BufferWrapper {
    pub device: Arc<DeviceWrapper>,
    pub buffer: api::VkBuffer,
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
            }),
            result => Err(VulkanError::VulkanError(result)),
        }
    }
    pub unsafe fn allocate_and_bind_memory(
        self,
        device_memory_pools: &DeviceMemoryPools,
        element_alignment: usize,
        preferred_properties: Option<api::VkMemoryPropertyFlags>,
        required_properties: api::VkMemoryPropertyFlags,
    ) -> Result<(Self, DeviceMemoryPoolAllocation)> {
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
            api::VK_SUCCESS => Ok((self, memory_allocation)),
            result => Err(VulkanError::VulkanError(result)),
        }
    }
}

pub struct VulkanStagingVertexBufferImplementation {
    pub staging_buffer: BufferWrapper,
    pub staging_device_memory: DeviceMemoryPoolAllocation,
    pub device_buffer: VulkanDeviceVertexBuffer,
}

pub struct VulkanStagingVertexBuffer(VulkanStagingVertexBufferImplementation);

pub fn into_vulkan_staging_vertex_buffer_implementation(
    v: VulkanStagingVertexBuffer,
) -> VulkanStagingVertexBufferImplementation {
    v.0
}

const STAGING_VERTEX_BUFFER_USAGE_FLAGS: api::VkBufferUsageFlags =
    api::VK_BUFFER_USAGE_TRANSFER_SRC_BIT;

pub unsafe fn create_staging_vertex_buffer(
    device: Arc<DeviceWrapper>,
    device_memory_pools: &DeviceMemoryPools,
    element_count: usize,
) -> Result<VulkanStagingVertexBuffer> {
    let buffer = BufferWrapper::new(
        device.clone(),
        element_count as u64 * mem::size_of::<VertexBufferElement>() as u64,
        STAGING_VERTEX_BUFFER_USAGE_FLAGS,
        api::VK_SHARING_MODE_EXCLUSIVE,
        &[],
    )?;
    let (buffer, device_memory) = buffer.allocate_and_bind_memory(
        device_memory_pools,
        mem::align_of::<VertexBufferElement>(),
        None,
        api::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | api::VK_MEMORY_PROPERTY_HOST_COHERENT_BIT,
    )?;
    Ok(VulkanStagingVertexBuffer(
        VulkanStagingVertexBufferImplementation {
            staging_buffer: buffer,
            staging_device_memory: device_memory,
            device_buffer: create_device_vertex_buffer(device, device_memory_pools, element_count)?,
        },
    ))
}

impl StagingVertexBuffer for VulkanStagingVertexBuffer {
    fn len(&self) -> usize {
        self.0.device_buffer.0.element_count
    }
    fn write(&mut self, index: usize, value: VertexBufferElement) {
        let memory_slice = self
            .0
            .staging_device_memory
            .get_mapped_memory()
            .unwrap()
            .as_ptr();
        assert!(
            unsafe { &*memory_slice }.len()
                >= self.0.device_buffer.0.element_count * mem::size_of::<VertexBufferElement>()
        );
        let memory_slice = unsafe {
            slice::from_raw_parts_mut(
                memory_slice as *mut u8 as *mut VertexBufferElement,
                self.0.device_buffer.0.element_count,
            )
        };
        memory_slice[index] = value;
    }
}

#[derive(Clone)]
pub struct VulkanDeviceVertexBufferImplementation {
    pub buffer: Arc<BufferWrapper>,
    pub device_memory: Arc<DeviceMemoryPoolAllocation>,
    pub submit_tracker: Option<CommandBufferSubmitTracker>,
    pub element_count: usize,
}

#[derive(Clone)]
pub struct VulkanDeviceVertexBuffer(VulkanDeviceVertexBufferImplementation);

pub fn get_mut_vulkan_device_vertex_buffer_implementation(
    v: &mut VulkanDeviceVertexBuffer,
) -> &mut VulkanDeviceVertexBufferImplementation {
    &mut v.0
}

pub fn into_vulkan_device_vertex_buffer_implementation(
    v: VulkanDeviceVertexBuffer,
) -> VulkanDeviceVertexBufferImplementation {
    v.0
}

const DEVICE_VERTEX_BUFFER_USAGE_FLAGS: api::VkBufferUsageFlags =
    api::VK_BUFFER_USAGE_TRANSFER_DST_BIT | api::VK_BUFFER_USAGE_VERTEX_BUFFER_BIT;

pub unsafe fn create_device_vertex_buffer(
    device: Arc<DeviceWrapper>,
    device_memory_pools: &DeviceMemoryPools,
    element_count: usize,
) -> Result<VulkanDeviceVertexBuffer> {
    let buffer = BufferWrapper::new(
        device,
        element_count as u64 * mem::size_of::<VertexBufferElement>() as u64,
        DEVICE_VERTEX_BUFFER_USAGE_FLAGS,
        api::VK_SHARING_MODE_EXCLUSIVE,
        &[],
    )?;
    let (buffer, device_memory) = buffer.allocate_and_bind_memory(
        device_memory_pools,
        1,
        None,
        api::VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
    )?;
    Ok(VulkanDeviceVertexBuffer(
        VulkanDeviceVertexBufferImplementation {
            buffer: Arc::new(buffer),
            device_memory: Arc::new(device_memory),
            submit_tracker: None,
            element_count: element_count,
        },
    ))
}

impl DeviceVertexBuffer for VulkanDeviceVertexBuffer {
    fn len(&self) -> usize {
        self.0.element_count
    }
}

pub struct VulkanStagingIndexBufferImplementation {
    pub staging_buffer: BufferWrapper,
    pub staging_device_memory: DeviceMemoryPoolAllocation,
    pub device_buffer: VulkanDeviceIndexBuffer,
}

pub struct VulkanStagingIndexBuffer(VulkanStagingIndexBufferImplementation);

pub fn into_vulkan_staging_index_buffer_implementation(
    v: VulkanStagingIndexBuffer,
) -> VulkanStagingIndexBufferImplementation {
    v.0
}

const STAGING_INDEX_BUFFER_USAGE_FLAGS: api::VkBufferUsageFlags =
    api::VK_BUFFER_USAGE_TRANSFER_SRC_BIT;

pub unsafe fn create_staging_index_buffer(
    device: Arc<DeviceWrapper>,
    device_memory_pools: &DeviceMemoryPools,
    element_count: usize,
) -> Result<VulkanStagingIndexBuffer> {
    let buffer = BufferWrapper::new(
        device.clone(),
        element_count as u64 * mem::size_of::<IndexBufferElement>() as u64,
        STAGING_INDEX_BUFFER_USAGE_FLAGS,
        api::VK_SHARING_MODE_EXCLUSIVE,
        &[],
    )?;
    let (buffer, device_memory) = buffer.allocate_and_bind_memory(
        device_memory_pools,
        mem::align_of::<IndexBufferElement>(),
        None,
        api::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | api::VK_MEMORY_PROPERTY_HOST_COHERENT_BIT,
    )?;
    Ok(VulkanStagingIndexBuffer(
        VulkanStagingIndexBufferImplementation {
            staging_buffer: buffer,
            staging_device_memory: device_memory,
            device_buffer: create_device_index_buffer(device, device_memory_pools, element_count)?,
        },
    ))
}

impl StagingIndexBuffer for VulkanStagingIndexBuffer {
    fn len(&self) -> usize {
        self.0.device_buffer.0.element_count
    }
    fn write(&mut self, index: usize, value: IndexBufferElement) {
        let memory_slice = self
            .0
            .staging_device_memory
            .get_mapped_memory()
            .unwrap()
            .as_ptr();
        assert!(
            unsafe { &*memory_slice }.len()
                >= self.0.device_buffer.0.element_count * mem::size_of::<IndexBufferElement>()
        );
        let memory_slice = unsafe {
            slice::from_raw_parts_mut(
                memory_slice as *mut u8 as *mut IndexBufferElement,
                self.0.device_buffer.0.element_count,
            )
        };
        memory_slice[index] = value;
    }
}

#[derive(Clone)]
pub struct VulkanDeviceIndexBufferImplementation {
    pub buffer: Arc<BufferWrapper>,
    pub device_memory: Arc<DeviceMemoryPoolAllocation>,
    pub submit_tracker: Option<CommandBufferSubmitTracker>,
    pub element_count: usize,
}

#[derive(Clone)]
pub struct VulkanDeviceIndexBuffer(VulkanDeviceIndexBufferImplementation);

pub fn get_mut_vulkan_device_index_buffer_implementation(
    v: &mut VulkanDeviceIndexBuffer,
) -> &mut VulkanDeviceIndexBufferImplementation {
    &mut v.0
}

pub fn into_vulkan_device_index_buffer_implementation(
    v: VulkanDeviceIndexBuffer,
) -> VulkanDeviceIndexBufferImplementation {
    v.0
}

const DEVICE_INDEX_BUFFER_USAGE_FLAGS: api::VkBufferUsageFlags =
    api::VK_BUFFER_USAGE_TRANSFER_DST_BIT | api::VK_BUFFER_USAGE_INDEX_BUFFER_BIT;

pub unsafe fn create_device_index_buffer(
    device: Arc<DeviceWrapper>,
    device_memory_pools: &DeviceMemoryPools,
    element_count: usize,
) -> Result<VulkanDeviceIndexBuffer> {
    let buffer = BufferWrapper::new(
        device,
        element_count as u64 * mem::size_of::<IndexBufferElement>() as u64,
        DEVICE_INDEX_BUFFER_USAGE_FLAGS,
        api::VK_SHARING_MODE_EXCLUSIVE,
        &[],
    )?;
    let (buffer, device_memory) = buffer.allocate_and_bind_memory(
        device_memory_pools,
        1,
        None,
        api::VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
    )?;
    Ok(VulkanDeviceIndexBuffer(
        VulkanDeviceIndexBufferImplementation {
            buffer: Arc::new(buffer),
            device_memory: Arc::new(device_memory),
            submit_tracker: None,
            element_count: element_count,
        },
    ))
}

impl DeviceIndexBuffer for VulkanDeviceIndexBuffer {
    fn len(&self) -> usize {
        self.0.element_count
    }
}
