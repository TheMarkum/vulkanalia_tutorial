#![allow(
    dead_code,
    unused_variables,
    clippy::too_many_arguments,
    clippy::unnecessary_wraps
)]

use std::time::Instant;

use anyhow::{anyhow, Result};
use log::*;
use uniform::descriptor;
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::{
    ExtDebugUtilsExtension, InstanceV1_0, KhrSurfaceExtension, KhrSwapchainExtension,
};
use vulkanalia::{window, Entry, Instance};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

mod drawing;
mod pipeline;
mod presentation;
mod setup;
mod texture;
mod uniform;
mod vertex;

const VALIDATION_ENABLED: bool = cfg!(debug_assertions);

const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

fn main() -> Result<()> {
    pretty_env_logger::init();

    // Window

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Vulkan Tutorial (Rust)")
        .with_resizable(true)
        .build(&event_loop)?;

    // App

    let mut app = unsafe { App::create(&window)? };
    let mut destroying = false;
    let mut minimized = false;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            // Render a frame if our Vulkan app is not being destroyed.
            Event::MainEventsCleared if !destroying && !minimized => {
                unsafe { app.render(&window) }.unwrap()
            }
            // Trigger re-render, if resized.
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                if size.width == 0 || size.height == 0 {
                    minimized = true;
                } else {
                    minimized = false;
                    app.resized = true;
                }

                info!("Resized window.");
            }
            // Destroy our Vulkan app.
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                destroying = true;
                *control_flow = ControlFlow::Exit;
                unsafe {
                    app.device.device_wait_idle().unwrap();
                }
                unsafe {
                    app.destroy();
                }
            }
            _ => {}
        }
    });
}

/// Our Vulkan app.
#[derive(Clone, Debug)]
struct App {
    entry: Entry,
    instance: Instance,
    data: AppData,
    device: Device,
    frame: usize,
    resized: bool,
    start: Instant,
}

impl App {
    /// Creates our Vulkan app.
    unsafe fn create(window: &Window) -> Result<Self> {
        let loader = LibloadingLoader::new(LIBRARY)?;
        let entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?;

        let mut data = AppData::default();
        let instance = setup::instance::create_instance(window, &entry, &mut data)?;

        data.surface = window::create_surface(&instance, &window, &window)?;

        setup::device::pick_physical_device(&instance, &mut data)?;
        let device = setup::device::create_logical_device(&entry, &instance, &mut data)?;

        presentation::swapchain::create_swapchain(window, &instance, &device, &mut data)?;
        presentation::swapchain::create_swapchain_image_views(&device, &mut data)?;

        pipeline::pipeline::create_render_pass(&instance, &device, &mut data)?;

        uniform::descriptor::create_descriptor_set_layout(&device, &mut data)?;

        pipeline::pipeline::create_pipeline(&device, &mut data)?;

        drawing::frame_buffer::create_framebuffers(&device, &mut data)?;
        drawing::command_buffer::create_command_pool(&instance, &device, &mut data)?;

        texture::image::create_texture_image(&instance, &device, &mut data)?;
        texture::image::create_texture_image_view(&device, &mut data)?;
        texture::image::create_texture_sampler(&device, &mut data)?;

        vertex::vertex::create_vertex_buffer(&instance, &device, &mut data)?;
        vertex::vertex::create_index_buffer(&instance, &device, &mut data)?;

        uniform::descriptor::create_uniform_buffers(&instance, &device, &mut data)?;
        uniform::descriptor::create_descriptor_pool(&device, &mut data)?;
        uniform::descriptor::create_descriptor_sets(&device, &mut data)?;

        drawing::command_buffer::create_command_buffers(&device, &mut data)?;

        drawing::render::create_sync_objects(&device, &mut data)?;

        Ok(Self {
            entry,
            instance,
            data,
            device,
            frame: 0,
            resized: false,
            start: Instant::now(),
        })
    }

    /// Renders a frame for our Vulkan app.
    unsafe fn render(&mut self, window: &Window) -> Result<()> {
        self.device.wait_for_fences(
            &[self.data.drawing_data.in_flight_fences[self.frame]],
            true,
            u64::MAX,
        )?;
        self.device
            .reset_fences(&[self.data.drawing_data.in_flight_fences[self.frame]])?;

        let image_index = self
            .device
            .acquire_next_image_khr(
                self.data.presentation_data.swapchain,
                u64::MAX,
                self.data.drawing_data.image_available_semaphores[self.frame],
                vk::Fence::null(),
            )?
            .0 as usize;

        if !self.data.drawing_data.images_in_flight[image_index as usize].is_null() {
            self.device.wait_for_fences(
                &[self.data.drawing_data.images_in_flight[image_index as usize]],
                true,
                u64::MAX,
            )?;
        }

        self.data.drawing_data.images_in_flight[image_index as usize] =
            self.data.drawing_data.in_flight_fences[self.frame];

        descriptor::update_uniform_buffer(self, image_index)?;

        let wait_semaphores = &[self.data.drawing_data.image_available_semaphores[self.frame]];

        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.data.drawing_data.command_buffers[image_index as usize]];

        let signal_semaphores = &[self.data.drawing_data.render_finished_semaphores[self.frame]];

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        self.device
            .reset_fences(&[self.data.drawing_data.in_flight_fences[self.frame]])?;

        self.device.queue_submit(
            self.data.setup_data.graphics_queue,
            &[submit_info],
            self.data.drawing_data.in_flight_fences[self.frame],
        )?;

        let swapchains = &[self.data.presentation_data.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        let result = self
            .device
            .queue_present_khr(self.data.setup_data.present_queue, &present_info);

        let changed = result == Ok(vk::SuccessCode::SUBOPTIMAL_KHR)
            || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR);

        if self.resized || changed {
            self.resized = false;
            presentation::swapchain::recreate_swapchain(self, window)?;
        } else if let Err(e) = result {
            return Err(anyhow!(e));
        }

        self.frame = (self.frame + 1) % drawing::render::MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }

    /// Destroys our Vulkan app.
    unsafe fn destroy(&mut self) {
        presentation::swapchain::destroy_swapchain(self);

        self.device
            .destroy_sampler(self.data.texture_data.texture_sampler, None);

        self.device
            .destroy_image_view(self.data.texture_data.texture_image_view, None);

        self.device
            .destroy_image(self.data.texture_data.texture_image, None);
        self.device
            .free_memory(self.data.texture_data.texture_image_memory, None);

        self.device
            .destroy_descriptor_set_layout(self.data.uniform_data.descriptor_set_layout, None);

        self.device
            .destroy_buffer(self.data.vertex_data.index_buffer, None);
        self.device
            .free_memory(self.data.vertex_data.index_buffer_memory, None);

        self.device
            .destroy_buffer(self.data.vertex_data.vertex_buffer, None);
        self.device
            .free_memory(self.data.vertex_data.vertex_buffer_memory, None);

        self.data
            .drawing_data
            .in_flight_fences
            .iter()
            .for_each(|f| self.device.destroy_fence(*f, None));

        self.data
            .drawing_data
            .render_finished_semaphores
            .iter()
            .for_each(|s| self.device.destroy_semaphore(*s, None));
        self.data
            .drawing_data
            .image_available_semaphores
            .iter()
            .for_each(|s| self.device.destroy_semaphore(*s, None));

        self.device
            .destroy_command_pool(self.data.drawing_data.command_pool, None);

        self.device.destroy_device(None);

        if VALIDATION_ENABLED {
            self.instance
                .destroy_debug_utils_messenger_ext(self.data.setup_data.messenger, None);
        }

        self.instance.destroy_surface_khr(self.data.surface, None);
        self.instance.destroy_instance(None);
    }
}

/// The Vulkan handles and associated properties used by our Vulkan app.
#[derive(Clone, Debug, Default)]
struct AppData {
    surface: vk::SurfaceKHR,
    setup_data: setup::SetupData,
    presentation_data: presentation::PresentationData,
    uniform_data: uniform::UniformData,
    pipeline_data: pipeline::PipelineData,
    drawing_data: drawing::DrawingData,
    vertex_data: vertex::VertexData,
    texture_data: texture::TextureData,
}

unsafe fn begin_single_time_commands(device: &Device, data: &AppData) -> Result<vk::CommandBuffer> {
    let info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(data.vertex_data.command_pool)
        .command_buffer_count(1);

    let command_buffer = device.allocate_command_buffers(&info)?[0];

    let info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    device.begin_command_buffer(command_buffer, &info)?;

    Ok(command_buffer)
}

unsafe fn end_single_time_commands(
    device: &Device,
    data: &AppData,
    command_buffer: vk::CommandBuffer,
) -> Result<()> {
    device.end_command_buffer(command_buffer)?;

    let command_buffers = &[command_buffer];
    let info = vk::SubmitInfo::builder().command_buffers(command_buffers);

    device.queue_submit(data.setup_data.transfer_queue, &[info], vk::Fence::null())?;
    device.queue_wait_idle(data.setup_data.transfer_queue)?;

    device.free_command_buffers(data.vertex_data.command_pool, &[command_buffer]);

    Ok(())
}

unsafe fn create_image_view(
    device: &Device,
    image: vk::Image,
    format: vk::Format,
) -> Result<vk::ImageView> {
    let subresource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1);

    let info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .view_type(vk::ImageViewType::_2D)
        .format(format)
        .subresource_range(subresource_range);

    Ok(device.create_image_view(&info, None)?)
}
