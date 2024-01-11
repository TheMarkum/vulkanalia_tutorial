use anyhow::Result;
use log::*;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::{KhrSurfaceExtension, KhrSwapchainExtension};
use winit::window::Window;

use crate::drawing::{command_buffer, frame_buffer};
use crate::pipeline::pipeline;
use crate::setup::device::queue_families;
use crate::uniform::descriptor;
use crate::{create_image_view, App, AppData};

pub const DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[vk::KHR_SWAPCHAIN_EXTENSION.name];

#[derive(Clone, Debug)]
pub struct SwapchainSupport {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapchainSupport {
    pub unsafe fn get(
        instance: &Instance,
        data: &AppData,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Self> {
        Ok(Self {
            capabilities: instance
                .get_physical_device_surface_capabilities_khr(physical_device, data.surface)?,
            formats: instance
                .get_physical_device_surface_formats_khr(physical_device, data.surface)?,
            present_modes: instance
                .get_physical_device_surface_present_modes_khr(physical_device, data.surface)?,
        })
    }
}

fn get_swapchain_surface_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
    formats
        .iter()
        .cloned()
        .find(|f| {
            f.format == vk::Format::B8G8R8A8_SRGB
                && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .unwrap_or_else(|| formats[0])
}

fn get_swapchain_present_mode(present_modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
    present_modes
        .iter()
        .cloned()
        .find(|m| *m == vk::PresentModeKHR::MAILBOX)
        .unwrap_or(vk::PresentModeKHR::FIFO)
}

fn get_swapchain_extent(window: &Window, capabilities: vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        capabilities.current_extent
    } else {
        let size = window.inner_size();
        let clamp = |min: u32, max: u32, v: u32| min.max(max.min(v));
        vk::Extent2D::builder()
            .width(clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
                size.width,
            ))
            .height(clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
                size.height,
            ))
            .build()
    }
}

pub unsafe fn create_swapchain(
    window: &Window,
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    let support = SwapchainSupport::get(instance, data, data.setup_data.physical_device)?;

    let surface_format = get_swapchain_surface_format(&support.formats);
    let present_mode = get_swapchain_present_mode(&support.present_modes);
    let extent = get_swapchain_extent(window, support.capabilities);

    data.presentation_data.swapchain_format = surface_format.format;
    data.presentation_data.swapchain_extent = extent;

    let mut image_count = support.capabilities.min_image_count + 1;
    if support.capabilities.max_image_count != 0
        && image_count > support.capabilities.max_image_count
    {
        image_count = support.capabilities.max_image_count;
    }

    let indices =
        queue_families::QueueFamilyIndices::get(instance, data, data.setup_data.physical_device)?;

    let mut queue_family_indices = vec![];
    let image_sharing_mode = if indices.transfer != indices.present {
        queue_family_indices.push(indices.transfer);
        queue_family_indices.push(indices.graphics);
        queue_family_indices.push(indices.present);
        vk::SharingMode::CONCURRENT
    } else {
        vk::SharingMode::EXCLUSIVE
    };

    let info = vk::SwapchainCreateInfoKHR::builder()
        .surface(data.surface)
        .min_image_count(image_count)
        .image_format(surface_format.format)
        .image_color_space(surface_format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(image_sharing_mode)
        .queue_family_indices(&queue_family_indices)
        .pre_transform(support.capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true)
        .old_swapchain(vk::SwapchainKHR::null());

    data.presentation_data.swapchain = device.create_swapchain_khr(&info, None)?;
    data.presentation_data.swapchain_images =
        device.get_swapchain_images_khr(data.presentation_data.swapchain)?;

    info!("Swapchain created.");
    Ok(())
}

pub unsafe fn create_swapchain_image_views(device: &Device, data: &mut AppData) -> Result<()> {
    data.presentation_data.swapchain_image_views = data
        .presentation_data
        .swapchain_images
        .iter()
        .map(|i| create_image_view(device, *i, data.presentation_data.swapchain_format))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}

pub unsafe fn recreate_swapchain(app: &mut App, window: &Window) -> Result<()> {
    app.device.device_wait_idle()?;
    destroy_swapchain(app);

    create_swapchain(window, &app.instance, &app.device, &mut app.data)?;
    create_swapchain_image_views(&app.device, &mut app.data)?;

    pipeline::create_render_pass(&app.instance, &app.device, &mut app.data)?;
    pipeline::create_pipeline(&app.device, &mut app.data)?;

    frame_buffer::create_framebuffers(&app.device, &mut app.data)?;

    descriptor::create_uniform_buffers(&app.instance, &app.device, &mut app.data)?;
    descriptor::create_descriptor_pool(&app.device, &mut app.data)?;
    descriptor::create_descriptor_sets(&app.device, &mut app.data)?;

    command_buffer::create_command_buffers(&app.device, &mut app.data)?;

    app.data.drawing_data.images_in_flight.resize(
        app.data.presentation_data.swapchain_images.len(),
        vk::Fence::null(),
    );

    info!("Swapchain re-created.");
    Ok(())
}

pub unsafe fn destroy_swapchain(app: &mut App) {
    app.device
        .destroy_descriptor_pool(app.data.uniform_data.descriptor_pool, None);

    app.data
        .uniform_data
        .uniform_buffers
        .iter()
        .for_each(|b| app.device.destroy_buffer(*b, None));
    app.data
        .uniform_data
        .uniform_buffers_memory
        .iter()
        .for_each(|m| app.device.free_memory(*m, None));

    app.data
        .drawing_data
        .framebuffers
        .iter()
        .for_each(|f| app.device.destroy_framebuffer(*f, None));

    app.device.free_command_buffers(
        app.data.drawing_data.command_pool,
        &app.data.drawing_data.command_buffers,
    );

    app.device
        .destroy_pipeline(app.data.pipeline_data.pipeline, None);

    app.device
        .destroy_pipeline_layout(app.data.pipeline_data.pipeline_layout, None);

    app.device
        .destroy_render_pass(app.data.pipeline_data.render_pass, None);

    app.data
        .presentation_data
        .swapchain_image_views
        .iter()
        .for_each(|v| app.device.destroy_image_view(*v, None));

    app.device
        .destroy_swapchain_khr(app.data.presentation_data.swapchain, None);
}
