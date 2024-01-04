use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::DebugUtilsMessengerEXT;

pub mod device;
pub mod instance;

#[derive(Clone, Debug, Default)]
pub struct SetupData {
    pub messenger: DebugUtilsMessengerEXT,
    pub physical_device: vk::PhysicalDevice,
    pub transfer_queue: vk::Queue,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
}
