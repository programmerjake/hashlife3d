use super::{api, null_or_zero, DeviceWrapper, Result, VulkanError};
use std::ptr::null;
use std::sync::Arc;

pub struct SemaphoreWrapper {
    pub device: Arc<DeviceWrapper>,
    pub semaphore: api::VkSemaphore,
}

impl SemaphoreWrapper {
    pub fn new(device: Arc<DeviceWrapper>) -> Result<Self> {
        let mut semaphore = null_or_zero();
        match unsafe {
            device.vkCreateSemaphore.unwrap()(
                device.device,
                &api::VkSemaphoreCreateInfo {
                    sType: api::VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
                    pNext: null(),
                    flags: 0,
                },
                null(),
                &mut semaphore,
            )
        } {
            api::VK_SUCCESS => Ok(SemaphoreWrapper {
                semaphore: semaphore,
                device: device,
            }),
            result => Err(VulkanError::VulkanError(result)),
        }
    }
}

impl Drop for SemaphoreWrapper {
    fn drop(&mut self) {
        unsafe {
            self.device.vkDestroySemaphore.unwrap()(self.device.device, self.semaphore, null());
        }
    }
}
