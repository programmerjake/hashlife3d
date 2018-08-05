use super::{
    api, null_or_zero, DeviceMemoryPoolAllocation, DeviceMemoryPools, DeviceWrapper, Result,
    VulkanError,
};
use std::mem;
use std::ptr::null;
use std::sync::Arc;

pub struct ImageWrapper {
    pub device: Arc<DeviceWrapper>,
    pub image: api::VkImage,
    pub destroy_on_drop: bool,
    pub device_memory: Option<DeviceMemoryPoolAllocation>,
}

impl ImageWrapper {
    pub fn new_depth(
        device: Arc<DeviceWrapper>,
        format: api::VkFormat,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let mut image = null_or_zero();
        match unsafe {
            device.vkCreateImage.unwrap()(
                device.device,
                &api::VkImageCreateInfo {
                    sType: api::VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO,
                    pNext: null(),
                    flags: 0,
                    imageType: api::VK_IMAGE_TYPE_2D,
                    format: format,
                    extent: api::VkExtent3D {
                        width: width,
                        height: height,
                        depth: 1,
                    },
                    mipLevels: 1,
                    arrayLayers: 1,
                    samples: api::VK_SAMPLE_COUNT_1_BIT,
                    tiling: api::VK_IMAGE_TILING_OPTIMAL,
                    usage: api::VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT,
                    sharingMode: api::VK_SHARING_MODE_EXCLUSIVE,
                    queueFamilyIndexCount: 0,
                    pQueueFamilyIndices: null(),
                    initialLayout: api::VK_IMAGE_LAYOUT_UNDEFINED,
                },
                null(),
                &mut image,
            )
        } {
            api::VK_SUCCESS => Ok(ImageWrapper {
                device: device,
                image: image,
                destroy_on_drop: true,
                device_memory: None,
            }),
            result => Err(VulkanError::VulkanError(result)),
        }
    }
    pub unsafe fn allocate_and_bind_memory(
        mut self,
        device_memory_pools: &DeviceMemoryPools,
        preferred_properties: Option<api::VkMemoryPropertyFlags>,
        required_properties: api::VkMemoryPropertyFlags,
    ) -> Result<Self> {
        assert!(self.device_memory.is_none());
        let mut memory_requirements = mem::zeroed();
        self.device.vkGetImageMemoryRequirements.unwrap()(
            self.device.device,
            self.image,
            &mut memory_requirements,
        );
        let memory_allocation = device_memory_pools.allocate_from_memory_requirements(
            memory_requirements,
            preferred_properties,
            required_properties,
        )?;
        match self.device.vkBindImageMemory.unwrap()(
            self.device.device,
            self.image,
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

impl Drop for ImageWrapper {
    fn drop(&mut self) {
        if self.destroy_on_drop {
            unsafe {
                self.device.vkDestroyImage.unwrap()(self.device.device, self.image, null());
            }
        }
    }
}

pub struct ImageViewWrapper {
    pub image: Arc<ImageWrapper>,
    pub image_view: api::VkImageView,
}

impl Drop for ImageViewWrapper {
    fn drop(&mut self) {
        unsafe {
            self.image.device.vkDestroyImageView.unwrap()(
                self.image.device.device,
                self.image_view,
                null(),
            );
        }
    }
}
