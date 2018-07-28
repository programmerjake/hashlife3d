use super::{api, Queue};

pub struct VulkanQueue {
    queue: api::VkQueue,
}

impl Queue for VulkanQueue {}
