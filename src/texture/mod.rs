use vulkanalia::prelude::v1_0::*;

pub mod image;
pub mod multisampling;

#[derive(Clone, Debug, Default)]
pub struct TextureData {
    pub mip_levels: u32,
    pub msaa_samples: vk::SampleCountFlags,
    pub texture_image: vk::Image,
    pub texture_image_memory: vk::DeviceMemory,
    pub texture_image_view: vk::ImageView,
    pub texture_sampler: vk::Sampler,
    pub depth_image: vk::Image,
    pub depth_image_memory: vk::DeviceMemory,
    pub depth_image_view: vk::ImageView,
    pub color_image: vk::Image,
    pub color_image_memory: vk::DeviceMemory,
    pub color_image_view: vk::ImageView,
}
