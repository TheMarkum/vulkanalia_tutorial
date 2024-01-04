use vulkanalia::prelude::v1_0::*;

pub mod vertex;

#[derive(Clone, Debug, Default)]
pub struct VertexData {
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
}
