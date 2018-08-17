use super::{api, null_or_zero, DescriptorSetLayoutWrapper, DeviceWrapper, Result, VulkanError};
use std::ptr::null;
use std::sync::Arc;

pub struct DescriptorPoolWrapper {
    pub device: Arc<DeviceWrapper>,
    pub descriptor_pool: api::VkDescriptorPool,
}

impl Drop for DescriptorPoolWrapper {
    fn drop(&mut self) {
        unsafe {
            self.device.vkDestroyDescriptorPool.unwrap()(
                self.device.device,
                self.descriptor_pool,
                null(),
            )
        }
    }
}

impl DescriptorPoolWrapper {
    pub unsafe fn new(
        device: Arc<DeviceWrapper>,
        max_sets: u32,
        pool_sizes: &[api::VkDescriptorPoolSize],
    ) -> Result<Self> {
        let mut descriptor_pool = null_or_zero();
        match device.vkCreateDescriptorPool.unwrap()(
            device.device,
            &api::VkDescriptorPoolCreateInfo {
                sType: api::VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO,
                pNext: null(),
                flags: 0,
                maxSets: max_sets,
                poolSizeCount: pool_sizes.len() as u32,
                pPoolSizes: pool_sizes.as_ptr(),
            },
            null(),
            &mut descriptor_pool,
        ) {
            api::VK_SUCCESS => Ok(Self {
                device: device,
                descriptor_pool: descriptor_pool,
            }),
            result => Err(VulkanError::VulkanError(result)),
        }
    }
}

pub struct DescriptorSetWrapper {
    pub descriptor_pool: DescriptorPoolWrapper,
    pub descriptor_set_layout: Arc<DescriptorSetLayoutWrapper>,
    pub descriptor_set: api::VkDescriptorSet,
}

impl Drop for DescriptorSetWrapper {
    fn drop(&mut self) {
        // descriptor_set automatically freed by vkDestroyDescriptorPool
    }
}

impl DescriptorSetWrapper {
    pub unsafe fn new(
        descriptor_pool: DescriptorPoolWrapper,
        descriptor_set_layout: Arc<DescriptorSetLayoutWrapper>,
    ) -> Result<Self> {
        assert_eq!(
            descriptor_set_layout.device.device,
            descriptor_pool.device.device
        );
        let mut descriptor_set = null_or_zero();
        match descriptor_pool.device.vkAllocateDescriptorSets.unwrap()(
            descriptor_pool.device.device,
            &api::VkDescriptorSetAllocateInfo {
                sType: api::VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
                pNext: null(),
                descriptorPool: descriptor_pool.descriptor_pool,
                descriptorSetCount: 1,
                pSetLayouts: &descriptor_set_layout.descriptor_set_layout,
            },
            &mut descriptor_set,
        ) {
            api::VK_SUCCESS => Ok(Self {
                descriptor_pool: descriptor_pool,
                descriptor_set_layout: descriptor_set_layout,
                descriptor_set: descriptor_set,
            }),
            result => Err(VulkanError::VulkanError(result)),
        }
    }
}
