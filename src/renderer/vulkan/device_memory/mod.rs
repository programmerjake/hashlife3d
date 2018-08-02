mod buddy_suballocation_algorithm;
use super::{api, null_or_zero, DeviceWrapper, Result, VulkanError};
use std::collections::{HashMap, HashSet};
use std::mem;
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
    pub unsafe fn new(
        device: Arc<DeviceWrapper>,
        size: api::VkDeviceSize,
        memory_type_index: u32,
    ) -> Result<Self> {
        let mut device_memory = null_or_zero();
        match unsafe {
            device.vkAllocateMemory.unwrap()(
                device.device,
                &api::VkMemoryAllocateInfo {
                    sType: api::VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
                    pNext: null(),
                    allocationSize: size,
                    memoryTypeIndex: memory_type_index,
                },
                null(),
                &mut device_memory,
            )
        } {
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
        match unsafe {
            self.device.vkMapMemory.unwrap()(
                self.device.device,
                self.device_memory,
                0,
                api::VK_WHOLE_SIZE as api::VkDeviceSize,
                0,
                &mut mapped_memory,
            )
        } {
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
}

enum DeviceMemoryNodeKind {
    User,
    Pool {
        suballocations: (Option<Box<DeviceMemoryNode>>, Option<Box<DeviceMemoryNode>>),
    },
}

impl DeviceMemoryNodeKind {
    fn is_user(&self) -> bool {
        if let DeviceMemoryNodeKind::User = self {
            true
        } else {
            false
        }
    }
    fn is_pool(&self) -> bool {
        if let DeviceMemoryNodeKind::Pool { .. } = self {
            true
        } else {
            false
        }
    }
    fn get_suballocations_mut(
        &mut self,
    ) -> Option<&mut (Option<Box<DeviceMemoryNode>>, Option<Box<DeviceMemoryNode>>)> {
        use self::DeviceMemoryNodeKind::*;
        match self {
            Pool { suballocations } => Some(suballocations),
            User => None,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
enum DeviceMemoryNodeOrChunk {
    Node(NonNull<DeviceMemoryNode>),
    Chunk(NonNull<DeviceMemoryChunk>),
}

impl DeviceMemoryNodeOrChunk {
    unsafe fn get_node<'a>(self) -> &'a mut DeviceMemoryNode {
        use self::DeviceMemoryNodeOrChunk::*;
        match self {
            Node(node) => &mut *node.as_ptr(),
            Chunk(chunk) => &mut (*chunk.as_ptr()).allocation,
        }
    }
    fn is_chunk(self) -> bool {
        if let DeviceMemoryNodeOrChunk::Chunk(_) = self {
            true
        } else {
            false
        }
    }
}

struct DeviceMemoryNode {
    offset: api::VkDeviceSize,
    log2_size: u8,
    parent: DeviceMemoryNodeOrChunk,
    kind: DeviceMemoryNodeKind,
}

struct DeviceMemoryChunk {
    device_memory: Arc<DeviceMemoryWrapper>,
    allocation: DeviceMemoryNode,
}

impl DeviceMemoryChunk {
    fn new(device_memory: Arc<DeviceMemoryWrapper>, log2_size: u8) -> Box<Self> {
        let mut retval = Box::new(Self {
            device_memory: device_memory,
            allocation: DeviceMemoryNode {
                offset: 0,
                log2_size: log2_size,
                parent: DeviceMemoryNodeOrChunk::Chunk(NonNull::dangling()),
                kind: DeviceMemoryNodeKind::Pool {
                    suballocations: (None, None),
                },
            },
        });
        retval.allocation.parent = DeviceMemoryNodeOrChunk::Chunk(retval.as_mut().into());
        retval
    }
}

struct DeviceMemoryPoolLockedState {
    chunks: HashMap<NonNull<DeviceMemoryChunk>, Box<DeviceMemoryChunk>>,
    nodes_with_free_children: Vec<HashSet<DeviceMemoryNodeOrChunk>>,
}

struct DeviceMemoryPool {
    device: Arc<DeviceWrapper>,
    memory_type_index: u32,
    state: Mutex<DeviceMemoryPoolLockedState>,
}

pub struct DeviceMemoryPoolRef(Arc<DeviceMemoryPool>);

unsafe impl Sync for DeviceMemoryPoolRef {}
unsafe impl Send for DeviceMemoryPoolRef {}

pub trait GenericDeviceMemoryPoolAllocation: Send {
    fn get_device_memory(&self) -> &Arc<DeviceMemoryWrapper>;
    fn get_offset(&self) -> api::VkDeviceSize;
    fn get_size(&self) -> api::VkDeviceSize;
}

pub trait GenericDeviceMemoryPoolRef: Sync + Send + Clone {
    type Allocation: GenericDeviceMemoryPoolAllocation;
    unsafe fn new(device: Arc<DeviceWrapper>, memory_type_index: u32) -> Self;
    fn allocate(
        &self,
        size: api::VkDeviceSize,
        alignment: api::VkDeviceSize,
    ) -> Result<Self::Allocation>;
}

impl DeviceMemoryPoolRef {
    pub unsafe fn new(device: Arc<DeviceWrapper>, memory_type_index: u32) -> Self {
        DeviceMemoryPoolRef(Arc::new(DeviceMemoryPool {
            device: device,
            memory_type_index: memory_type_index,
            state: Mutex::new(DeviceMemoryPoolLockedState {
                chunks: HashMap::new(),
                nodes_with_free_children: Vec::new(),
            }),
        }))
    }
    pub fn allocate(
        &self,
        size: api::VkDeviceSize,
        alignment: api::VkDeviceSize,
    ) -> Result<DeviceMemoryPoolAllocation> {
        assert!(size != 0 && alignment != 0);
        unimplemented!()
    }
}

pub struct DeviceMemoryPoolAllocation {
    pool: Arc<DeviceMemoryPool>,
    device_memory: Arc<DeviceMemoryWrapper>,
    offset: api::VkDeviceSize,
    size: api::VkDeviceSize,
    chunk: NonNull<DeviceMemoryChunk>,
    node: DeviceMemoryNodeOrChunk,
}

impl GenericDeviceMemoryPoolAllocation for DeviceMemoryPoolAllocation {
    fn get_device_memory(&self) -> &Arc<DeviceMemoryWrapper> {
        unimplemented!()
    }
    fn get_offset(&self) -> api::VkDeviceSize {
        unimplemented!()
    }
    fn get_size(&self) -> api::VkDeviceSize {
        unimplemented!()
    }
}

unsafe impl Send for DeviceMemoryPoolAllocation {}

impl Drop for DeviceMemoryPoolAllocation {
    fn drop(&mut self) {
        unimplemented!()
        /*
        let state = self.pool.state.lock().unwrap();
        unsafe {
            let mut node = self.node;
            assert!(node.get_node().kind.is_user());
            node.get_node().kind = DeviceMemoryNodeKind::Pool {
                suballocations: (None, None),
            };
            loop {
                let parent = node.get_node().parent;
                let parent_suballocations =
                    parent.get_node().kind.get_suballocations_mut().unwrap();
                let (my_suballocation, other_suballocation) = parent_suballocations;
                if (node.get_node().offset & (1 << node.get_node().log2_size)) != 0 {
                    mem::swap(&mut my_suballocation, &mut other_suballocation);
                }
                *my_suballocation = None;
                node = parent;
                if other_suballocation.is_none() && !node.is_chunk() {
                    state.nodes_with_free_children[node.get_node().log2_size as usize]
                        .remove(&node);
                } else {
                    unimplemented!();
                    /* state.nodes_with_free_children[node.get_node().log2_size as usize]
                        .insert(&node); */
                    break;
                }
            }
        }
        mem::drop(state);*/
    }
}
