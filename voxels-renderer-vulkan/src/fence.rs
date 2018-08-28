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
use renderer::{Fence, FenceTryWaitResult};
use std::any::Any;
use std::ptr::null;
use std::sync::{atomic::*, Arc, Mutex};
use std::u64;

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
    pub fn wait(&mut self) -> Result<()> {
        match unsafe {
            self.device.vkWaitForFences.unwrap()(
                self.device.device,
                1,
                &self.fence,
                api::VK_TRUE,
                u64::MAX,
            )
        } {
            api::VK_SUCCESS => Ok(()),
            result => Err(VulkanError::VulkanError(result)),
        }
    }
    pub fn try_wait(&mut self) -> Result<FenceTryWaitResult> {
        match unsafe { self.device.vkGetFenceStatus.unwrap()(self.device.device, self.fence) } {
            api::VK_SUCCESS => Ok(FenceTryWaitResult::Ready),
            api::VK_NOT_READY => Ok(FenceTryWaitResult::WouldBlock),
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

pub struct VulkanFenceVulkanState {
    pub fence: FenceWrapper,
    pub referenced_objects: Vec<Box<Any + Send + Sync + 'static>>,
}

impl VulkanFenceVulkanState {
    pub fn clear_if_done(this: &mut Option<VulkanFenceVulkanState>) -> Result<FenceTryWaitResult> {
        let retval = match this {
            None => return Ok(FenceTryWaitResult::Ready),
            Some(VulkanFenceVulkanState { fence, .. }) => fence.try_wait(),
        };
        if let Ok(FenceTryWaitResult::Ready) = &retval {
            *this = None;
        }
        retval
    }
}

pub struct VulkanFenceState {
    vulkan_state: Mutex<Option<VulkanFenceVulkanState>>,
    wait_completed: Arc<AtomicBool>,
}

#[derive(Clone)]
pub struct VulkanFence(Arc<VulkanFenceState>);

pub fn create_fence(device: Arc<DeviceWrapper>) -> Result<VulkanFence> {
    Ok(VulkanFence(Arc::new(VulkanFenceState {
        vulkan_state: Mutex::new(Some(VulkanFenceVulkanState {
            fence: FenceWrapper::new(device, FenceState::Unsignaled)?,
            referenced_objects: Vec::new(),
        })),
        wait_completed: Arc::new(AtomicBool::new(false)),
    })))
}

pub fn create_signaled_fence() -> VulkanFence {
    VulkanFence(Arc::new(VulkanFenceState {
        vulkan_state: Mutex::new(None),
        wait_completed: Arc::new(AtomicBool::new(false)),
    }))
}

pub fn get_fence_wait_completed(fence: &VulkanFence) -> &Arc<AtomicBool> {
    &fence.0.wait_completed
}

pub fn get_fence_vulkan_state(fence: &VulkanFence) -> &Mutex<Option<VulkanFenceVulkanState>> {
    &fence.0.vulkan_state
}

impl Fence for VulkanFence {
    type Error = VulkanError;
    fn try_wait(&self) -> Result<FenceTryWaitResult> {
        let retval =
            VulkanFenceVulkanState::clear_if_done(&mut *self.0.vulkan_state.lock().unwrap());
        if let Ok(FenceTryWaitResult::Ready) = &retval {
            self.0.wait_completed.store(true, Ordering::Release);
        }
        retval
    }
    fn wait(self) -> Result<()> {
        let mut locked_state = self.0.vulkan_state.lock().unwrap();
        match &mut *locked_state {
            None => return Ok(()),
            Some(VulkanFenceVulkanState { fence, .. }) => fence.wait()?,
        };
        *locked_state = None;
        self.0.wait_completed.store(true, Ordering::Release);
        Ok(())
    }
}
