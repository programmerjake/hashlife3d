use super::{api, null_or_zero, DeviceWrapper, Result, VulkanError};
use std::ptr::null;
use std::sync::Arc;

pub struct FenceWrapper {
    pub device: Arc<DeviceWrapper>,
    pub fence: api::VkFence,
}

#[allow(dead_code)]
pub enum FenceState {
    Unsignaled,
    Signaled,
}

impl FenceWrapper {
    pub fn new(device: Arc<DeviceWrapper>, initial_state: FenceState) -> Result<Self> {
        let mut fence = null_or_zero();
        match unsafe {
            device.vkCreateFence.unwrap()(
                device.device,
                &api::VkFenceCreateInfo {
                    sType: api::VK_STRUCTURE_TYPE_FENCE_CREATE_INFO,
                    pNext: null(),
                    flags: match initial_state {
                        FenceState::Unsignaled => 0,
                        FenceState::Signaled => api::VK_FENCE_CREATE_SIGNALED_BIT,
                    },
                },
                null(),
                &mut fence,
            )
        } {
            api::VK_SUCCESS => Ok(FenceWrapper {
                fence: fence,
                device: device,
            }),
            result => Err(VulkanError::VulkanError(result)),
        }
    }
}

impl Drop for FenceWrapper {
    fn drop(&mut self) {
        unsafe {
            self.device.vkDestroyFence.unwrap()(self.device.device, self.fence, null());
        }
    }
}
