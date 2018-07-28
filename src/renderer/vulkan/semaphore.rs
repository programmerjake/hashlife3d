use super::Semaphore;

pub struct VulkanSemaphore {}

impl Drop for VulkanSemaphore {
    fn drop(&mut self) {
        unimplemented!()
    }
}

impl Semaphore for VulkanSemaphore {}
