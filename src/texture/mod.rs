use vulkanalia::prelude::v1_0::*;

pub mod image;

#[derive(Clone, Debug, Default)]
pub struct TextureData {
    pub texture_image: vk::Image,
    pub texture_image_memory: vk::DeviceMemory,
    pub texture_image_view: vk::ImageView,
    pub texture_sampler: vk::Sampler,
}
