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
