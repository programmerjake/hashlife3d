use super::{api, DeviceWrapper, Fence};
use std::ptr::null;
use std::sync::Arc;

pub struct VulkanFence {
    pub device: Arc<DeviceWrapper>,
    pub fence: api::VkFence,
}

unsafe impl Send for VulkanFence {}

impl Fence for VulkanFence {}

impl VulkanFence {
    pub fn get(&self) -> api::VkFence {
        self.fence
    }
}

impl Drop for VulkanFence {
    fn drop(&mut self) {
        unsafe {
            self.device.vkDestroyFence.unwrap()(self.device.device, self.fence, null());
        }
    }
}
