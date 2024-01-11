use std::mem::size_of;
use std::ptr::copy_nonoverlapping as memcpy;

use anyhow::Result;
use cgmath::{point3, vec3, Deg};
use vulkanalia::prelude::v1_0::*;

use crate::vertex::vertex;
use crate::App;
use crate::AppData;

type Mat4 = cgmath::Matrix4<f32>;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct UniformBufferObject {
    model: Mat4,
    view: Mat4,
    proj: Mat4,
}

pub unsafe fn create_descriptor_set_layout(device: &Device, data: &mut AppData) -> Result<()> {
    let ubo_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX);

    let sampler_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT);

    let bindings = &[ubo_binding, sampler_binding];
    let info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(bindings);

    data.uniform_data.descriptor_set_layout = device.create_descriptor_set_layout(&info, None)?;

    Ok(())
}

pub unsafe fn create_uniform_buffers(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    data.uniform_data.uniform_buffers.clear();
    data.uniform_data.uniform_buffers_memory.clear();

    for _ in 0..data.presentation_data.swapchain_images.len() {
        let (uniform_buffer, uniform_buffer_memory) = vertex::create_buffer(
            instance,
            device,
            data,
            size_of::<UniformBufferObject>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        data.uniform_data.uniform_buffers.push(uniform_buffer);
        data.uniform_data
            .uniform_buffers_memory
            .push(uniform_buffer_memory);
    }

    Ok(())
}

pub unsafe fn update_uniform_buffer(app: &App, image_index: usize) -> Result<()> {
    let time = app.start.elapsed().as_secs_f32();

    let model = Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), Deg(90.0) * time);

    let view = Mat4::look_at_rh(
        point3(2.0, 2.0, 2.0),
        point3(0.0, 0.0, 0.0),
        vec3(0.0, 0.0, 1.0),
    );

    let mut proj = cgmath::perspective(
        Deg(45.0),
        app.data.presentation_data.swapchain_extent.width as f32
            / app.data.presentation_data.swapchain_extent.height as f32,
        0.1,
        10.0,
    );

    proj[1][1] *= -1.0;

    let ubo = UniformBufferObject { model, view, proj };

    let memory = app.device.map_memory(
        app.data.uniform_data.uniform_buffers_memory[image_index],
        0,
        size_of::<UniformBufferObject>() as u64,
        vk::MemoryMapFlags::empty(),
    )?;

    memcpy(&ubo, memory.cast(), 1);

    app.device
        .unmap_memory(app.data.uniform_data.uniform_buffers_memory[image_index]);

    Ok(())
}

pub unsafe fn create_descriptor_pool(device: &Device, data: &mut AppData) -> Result<()> {
    let ubo_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(data.presentation_data.swapchain_images.len() as u32);

    let sampler_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(data.presentation_data.swapchain_images.len() as u32);

    let pool_sizes = &[ubo_size, sampler_size];
    let info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(pool_sizes)
        .max_sets(data.presentation_data.swapchain_images.len() as u32);

    data.uniform_data.descriptor_pool = device.create_descriptor_pool(&info, None)?;

    Ok(())
}

pub unsafe fn create_descriptor_sets(device: &Device, data: &mut AppData) -> Result<()> {
    let layouts = vec![
        data.uniform_data.descriptor_set_layout;
        data.presentation_data.swapchain_images.len()
    ];
    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(data.uniform_data.descriptor_pool)
        .set_layouts(&layouts);

    data.uniform_data.descriptor_sets = device.allocate_descriptor_sets(&info)?;

    for i in 0..data.presentation_data.swapchain_images.len() {
        let info = vk::DescriptorBufferInfo::builder()
            .buffer(data.uniform_data.uniform_buffers[i])
            .offset(0)
            .range(size_of::<UniformBufferObject>() as u64);

        let buffer_info = &[info];
        let ubo_write = vk::WriteDescriptorSet::builder()
            .dst_set(data.uniform_data.descriptor_sets[i])
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(buffer_info);

        let info = vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(data.texture_data.texture_image_view)
            .sampler(data.texture_data.texture_sampler);

        let image_info = &[info];
        let sampler_write = vk::WriteDescriptorSet::builder()
            .dst_set(data.uniform_data.descriptor_sets[i])
            .dst_binding(1)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(image_info);

        device.update_descriptor_sets(&[ubo_write, sampler_write], &[] as &[vk::CopyDescriptorSet]);
    }

    Ok(())
}
