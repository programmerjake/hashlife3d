use super::{api, Queue};

pub struct VulkanQueue {
    pub queue: api::VkQueue,
}

impl Queue for VulkanQueue {}
