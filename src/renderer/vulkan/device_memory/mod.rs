mod buddy_suballocation_algorithm;
use self::buddy_suballocation_algorithm::BuddySuballocationAlgorithm as SelectedSuballocationAlgorithm;
use super::{api, null_or_zero, DeviceWrapper, Result, VulkanError};
use std::cmp;
use std::ptr::*;
use std::result;
use std::slice;
use std::sync::{Arc, Mutex};

trait Suballocation: 'static + Send {
    fn get_offset(&self) -> u64;
    fn get_size(&self) -> u64;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct SuballocationFailed;

trait SuballocationAlgorithm: Send {
    type Suballocation: Suballocation;
    fn new(size: u64) -> Self;
    fn allocate(
        &mut self,
        size: u64,
        alignment: u64,
    ) -> result::Result<Self::Suballocation, SuballocationFailed>;
    fn free(&mut self, suballocation: Self::Suballocation);
}

type SelectedSuballocation =
    <SelectedSuballocationAlgorithm as SuballocationAlgorithm>::Suballocation;

pub struct DeviceMemoryWrapper {
    device: Arc<DeviceWrapper>,
    device_memory: api::VkDeviceMemory,
    size: api::VkDeviceSize,
    mapped_memory: Option<NonNull<[u8]>>,
}

unsafe impl Send for DeviceMemoryWrapper {}
unsafe impl Sync for DeviceMemoryWrapper {}

impl Drop for DeviceMemoryWrapper {
    fn drop(&mut self) {
        self.mapped_memory = None;
        unsafe {
            self.device.vkFreeMemory.unwrap()(self.device.device, self.device_memory, null());
        }
    }
}

impl DeviceMemoryWrapper {
    pub fn get_device_memory(&self) -> api::VkDeviceMemory {
        self.device_memory
    }
    pub unsafe fn new(
        device: Arc<DeviceWrapper>,
        size: api::VkDeviceSize,
        memory_type_index: u32,
    ) -> Result<Self> {
        let mut device_memory = null_or_zero();
        match device.vkAllocateMemory.unwrap()(
            device.device,
            &api::VkMemoryAllocateInfo {
                sType: api::VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
                pNext: null(),
                allocationSize: size,
                memoryTypeIndex: memory_type_index,
            },
            null(),
            &mut device_memory,
        ) {
            api::VK_SUCCESS => Ok(Self {
                device: device,
                device_memory: device_memory,
                size: size,
                mapped_memory: None,
            }),
            result => Err(VulkanError::VulkanError(result)),
        }
    }
    pub fn get_mapped_memory(&self) -> Option<NonNull<[u8]>> {
        self.mapped_memory
    }
    pub unsafe fn map_memory(&mut self) -> Result<NonNull<[u8]>> {
        assert!(self.mapped_memory.is_none());
        let mut mapped_memory = null_mut();
        match self.device.vkMapMemory.unwrap()(
            self.device.device,
            self.device_memory,
            0,
            api::VK_WHOLE_SIZE as api::VkDeviceSize,
            0,
            &mut mapped_memory,
        ) {
            api::VK_SUCCESS => {
                let retval =
                    slice::from_raw_parts_mut(mapped_memory as *mut u8, self.size as usize).into();
                self.mapped_memory = Some(retval);
                Ok(retval)
            }
            result => Err(VulkanError::VulkanError(result)),
        }
    }
    pub fn unmap_memory(&mut self) {
        assert!(self.mapped_memory.is_some());
        self.mapped_memory = None;
        unsafe { self.device.vkUnmapMemory.unwrap()(self.device.device, self.device_memory) };
    }
    pub fn get_size(&self) -> api::VkDeviceSize {
        self.size
    }
}

struct DeviceMemoryChunk {
    device_memory: Arc<DeviceMemoryWrapper>,
    suballocation_algorithm: Arc<Mutex<SelectedSuballocationAlgorithm>>,
}

struct DeviceMemoryPoolLockedState {
    chunks: Vec<DeviceMemoryChunk>,
    next_chunk: usize,
}

struct DeviceMemoryPool {
    device: Arc<DeviceWrapper>,
    memory_type_index: u32,
    must_be_mapped: bool,
    state: Mutex<DeviceMemoryPoolLockedState>,
}

pub struct DeviceMemoryPoolRef(Arc<DeviceMemoryPool>);

unsafe impl Sync for DeviceMemoryPoolRef {}
unsafe impl Send for DeviceMemoryPoolRef {}

struct SuballocationState {
    suballocation_algorithm: Arc<Mutex<SelectedSuballocationAlgorithm>>,
    suballocation: SelectedSuballocation,
}

pub struct DeviceMemoryPoolAllocation {
    device_memory: Arc<DeviceMemoryWrapper>,
    suballocation_state: Option<SuballocationState>,
}

impl DeviceMemoryPoolAllocation {
    pub fn get_device_memory(&self) -> &Arc<DeviceMemoryWrapper> {
        &self.device_memory
    }
    pub fn get_offset(&self) -> api::VkDeviceSize {
        match &self.suballocation_state {
            Some(SuballocationState { suballocation, .. }) => suballocation.get_offset(),
            None => 0,
        }
    }
    pub fn get_size(&self) -> api::VkDeviceSize {
        match &self.suballocation_state {
            Some(SuballocationState { suballocation, .. }) => suballocation.get_size(),
            None => self.device_memory.get_size(),
        }
    }
    pub fn get_mapped_memory(&self) -> Option<NonNull<[u8]>> {
        let mapped_memory = match self.device_memory.get_mapped_memory() {
            Some(m) => unsafe { &mut *m.as_ptr() },
            None => return None,
        };
        let offset = self.get_offset() as usize;
        let size = self.get_size() as usize;
        Some(mapped_memory[offset..][..size].into())
    }
}

impl Drop for DeviceMemoryPoolAllocation {
    fn drop(&mut self) {
        let SuballocationState {
            suballocation_algorithm,
            suballocation,
        } = self.suballocation_state.take().unwrap();
        suballocation_algorithm.lock().unwrap().free(suballocation);
    }
}

const ALLOCATION_MIN_CHUNK_SIZE: api::VkDeviceSize = 16 << 20; // 16 MiB
const ALLOCATION_MAX_NONDEDICATED_CHUNK_SIZE: api::VkDeviceSize = 128 << 20; // 128 MiB

impl DeviceMemoryPoolRef {
    pub unsafe fn new(
        device: Arc<DeviceWrapper>,
        memory_type_index: u32,
        must_be_mapped: bool,
    ) -> Self {
        DeviceMemoryPoolRef(Arc::new(DeviceMemoryPool {
            device: device,
            memory_type_index: memory_type_index,
            must_be_mapped: must_be_mapped,
            state: Mutex::new(DeviceMemoryPoolLockedState {
                chunks: Vec::new(),
                next_chunk: 0,
            }),
        }))
    }
    pub fn allocate(
        &self,
        mut size: api::VkDeviceSize,
        alignment: api::VkDeviceSize,
    ) -> Result<DeviceMemoryPoolAllocation> {
        assert!(size != 0 && alignment != 0);
        assert!(alignment.is_power_of_two());
        size = cmp::max(size, alignment);
        size = size.checked_next_power_of_two().unwrap();
        let mut locked_state = self.0.state.lock().unwrap();
        let DeviceMemoryPoolLockedState { chunks, next_chunk } = &mut *locked_state;
        for _ in 0..chunks.len() {
            if *next_chunk >= chunks.len() {
                *next_chunk = 0;
            }
            let chunk = &chunks[*next_chunk];
            if chunk.device_memory.get_size() < size {
                match chunk
                    .suballocation_algorithm
                    .lock()
                    .unwrap()
                    .allocate(size, alignment)
                {
                    Ok(suballocation) => {
                        return Ok(DeviceMemoryPoolAllocation {
                            device_memory: chunk.device_memory.clone(),
                            suballocation_state: Some(SuballocationState {
                                suballocation_algorithm: chunk.suballocation_algorithm.clone(),
                                suballocation: suballocation,
                            }),
                        });
                    }
                    Err(SuballocationFailed {}) => (),
                }
            }
            *next_chunk = *next_chunk + 1;
        }
        *next_chunk = chunks.len();
        assert!(ALLOCATION_MIN_CHUNK_SIZE > 1);
        let mut new_chunk_size = chunks
            .last()
            .map(|chunk| chunk.device_memory.get_size() * 2)
            .unwrap_or(ALLOCATION_MIN_CHUNK_SIZE);
        new_chunk_size = cmp::max(new_chunk_size, ALLOCATION_MAX_NONDEDICATED_CHUNK_SIZE);
        new_chunk_size = cmp::min(new_chunk_size, size);
        let mut device_memory = unsafe {
            DeviceMemoryWrapper::new(
                self.0.device.clone(),
                new_chunk_size,
                self.0.memory_type_index,
            )
        }?;
        if self.0.must_be_mapped {
            unsafe {
                device_memory.map_memory()?;
            }
        }
        let device_memory = Arc::new(device_memory);
        let mut suballocation_algorithm = SelectedSuballocationAlgorithm::new(new_chunk_size);
        let suballocation = suballocation_algorithm.allocate(size, alignment).unwrap();
        let suballocation_algorithm = Arc::new(Mutex::new(suballocation_algorithm));
        chunks.push(DeviceMemoryChunk {
            device_memory: device_memory.clone(),
            suballocation_algorithm: suballocation_algorithm.clone(),
        });
        Ok(DeviceMemoryPoolAllocation {
            device_memory: device_memory,
            suballocation_state: Some(SuballocationState {
                suballocation_algorithm: suballocation_algorithm,
                suballocation: suballocation,
            }),
        })
    }
}

unsafe impl Send for DeviceMemoryPoolAllocation {}

pub struct DeviceMemoryPools {
    memory_pools: Vec<DeviceMemoryPoolRef>,
    memory_properties: api::VkPhysicalDeviceMemoryProperties,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct NoMatchingMemoryType;

impl DeviceMemoryPools {
    pub unsafe fn new(
        device: Arc<DeviceWrapper>,
        memory_properties: api::VkPhysicalDeviceMemoryProperties,
    ) -> Self {
        assert!(memory_properties.memoryTypeCount <= api::VK_MAX_MEMORY_TYPES as u32);
        let mut memory_pools = Vec::with_capacity(memory_properties.memoryTypeCount as usize);
        for memory_type_index in 0..memory_properties.memoryTypeCount {
            memory_pools.push(DeviceMemoryPoolRef::new(
                device.clone(),
                memory_type_index,
                (memory_properties.memoryTypes[memory_type_index as usize].propertyFlags
                    & api::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT)
                    != 0,
            ));
        }
        Self {
            memory_pools: memory_pools,
            memory_properties: memory_properties,
        }
    }
    fn get_matching_memory_type_index_helper(
        &self,
        memory_type_bits: u32,
        required_properties: api::VkMemoryPropertyFlags,
    ) -> Option<u32> {
        assert_eq!(
            memory_type_bits >> self.memory_properties.memoryTypeCount,
            0
        );
        for memory_type_index in 0..self.memory_properties.memoryTypeCount {
            if (memory_type_bits & (1 << memory_type_index)) == 0 {
                continue;
            }
            if (self.memory_properties.memoryTypes[memory_type_index as usize].propertyFlags
                & required_properties)
                == required_properties
            {
                return Some(memory_type_index);
            }
        }
        None
    }
    pub fn get_matching_memory_type_index(
        &self,
        memory_type_bits: u32,
        preferred_properties: Option<api::VkMemoryPropertyFlags>,
        required_properties: api::VkMemoryPropertyFlags,
    ) -> result::Result<u32, NoMatchingMemoryType> {
        preferred_properties
            .map_or(None, |preferred_properties| {
                self.get_matching_memory_type_index_helper(memory_type_bits, preferred_properties)
            }).or_else(|| {
                self.get_matching_memory_type_index_helper(memory_type_bits, required_properties)
            }).ok_or(NoMatchingMemoryType {})
    }
    pub fn allocate_from_memory_requirements(
        &self,
        memory_requirements: api::VkMemoryRequirements,
        preferred_properties: Option<api::VkMemoryPropertyFlags>,
        required_properties: api::VkMemoryPropertyFlags,
    ) -> Result<DeviceMemoryPoolAllocation> {
        let memory_type_index = self.get_matching_memory_type_index(
            memory_requirements.memoryTypeBits,
            preferred_properties,
            required_properties,
        )?;
        self.memory_pools[memory_type_index as usize]
            .allocate(memory_requirements.size, memory_requirements.alignment)
    }
}
