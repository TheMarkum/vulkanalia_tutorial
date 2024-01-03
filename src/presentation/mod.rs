use vulkanalia::prelude::v1_0::*;

pub mod swapchain;

#[derive(Clone, Debug, Default)]
pub struct PresentationData {
    pub swapchain_format: vk::Format,
    pub swapchain_extent: vk::Extent2D,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_image_views: Vec<vk::ImageView>,
}