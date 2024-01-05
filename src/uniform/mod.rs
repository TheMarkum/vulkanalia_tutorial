use vulkanalia::prelude::v1_0::*;

pub mod descriptor;

#[derive(Clone, Debug, Default)]
pub struct UniformData {
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub uniform_buffers: Vec<vk::Buffer>,
    pub uniform_buffers_memory: Vec<vk::DeviceMemory>,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
}
