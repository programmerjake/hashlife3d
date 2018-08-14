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
use super::{api, InstanceWrapper, Result};
use sdl;
use std::ptr::{null, null_mut};
use std::rc::Rc;
use std::sync::Arc;

pub struct SurfaceWrapper {
    pub window: sdl::window::Window,
    pub instance: Arc<InstanceWrapper>,
    pub surface: api::VkSurfaceKHR,
}

impl Drop for SurfaceWrapper {
    fn drop(&mut self) {
        unsafe {
            self.instance.vkDestroySurfaceKHR.unwrap()(
                self.instance.instance,
                self.surface,
                null(),
            );
        }
    }
}

impl SurfaceWrapper {
    pub unsafe fn new(window: sdl::window::Window, instance: Arc<InstanceWrapper>) -> Result<Self> {
        let mut surface = null_mut();
        if sdl::api::SDL_Vulkan_CreateSurface(
            window.get(),
            instance.instance as sdl::api::VkInstance,
            &mut surface,
        ) == 0
        {
            Err(sdl::get_error().into())
        } else {
            Ok(Self {
                window: window,
                instance: instance,
                surface: surface as api::VkSurfaceKHR,
            })
        }
    }
}

pub struct SurfaceState {
    pub surface: Rc<SurfaceWrapper>,
    pub physical_device: api::VkPhysicalDevice,
    pub present_queue_index: u32,
    pub render_queue_index: u32,
    pub surface_format: api::VkSurfaceFormatKHR,
    pub depth_format: api::VkFormat,
    pub present_mode: api::VkPresentModeKHR,
    pub swapchain_desired_image_count: u32,
    pub swapchain_pre_transform: api::VkSurfaceTransformFlagBitsKHR,
    pub swapchain_composite_alpha: api::VkCompositeAlphaFlagBitsKHR,
    pub physical_device_memory_properties: api::VkPhysicalDeviceMemoryProperties,
}
