use super::{api, DeviceWrapper};
use renderer::{
    DeviceIndexBuffer, DeviceVertexBuffer, IndexBufferElement, StagingIndexBuffer,
    StagingVertexBuffer, VertexBufferElement,
};
use std::ptr::null;
use std::sync::Arc;

pub struct BufferWrapper {
    pub device: Arc<DeviceWrapper>,
    pub buffer: api::VkBuffer,
}

impl Drop for BufferWrapper {
    fn drop(&mut self) {
        unsafe {
            self.device.vkDestroyBuffer.unwrap()(self.device.device, self.buffer, null());
        }
    }
}

pub struct VulkanStagingVertexBuffer {}

impl StagingVertexBuffer for VulkanStagingVertexBuffer {
    fn len(&self) -> usize {
        unimplemented!()
    }
    fn write(&mut self, index: usize, value: VertexBufferElement) {
        unimplemented!()
    }
}

#[derive(Clone)]
pub struct VulkanDeviceVertexBuffer {}

impl DeviceVertexBuffer for VulkanDeviceVertexBuffer {}

pub struct VulkanStagingIndexBuffer {}

impl StagingIndexBuffer for VulkanStagingIndexBuffer {
    fn len(&self) -> usize {
        unimplemented!()
    }
    fn write(&mut self, index: usize, value: IndexBufferElement) {
        unimplemented!()
    }
}

#[derive(Clone)]
pub struct VulkanDeviceIndexBuffer {}

impl DeviceIndexBuffer for VulkanDeviceIndexBuffer {}
