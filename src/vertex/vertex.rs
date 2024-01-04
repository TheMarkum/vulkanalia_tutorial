use std::mem::size_of;
use std::ptr::copy_nonoverlapping as memcpy;

use anyhow::Result;
use cgmath::{vec2, vec3};
use vulkanalia::prelude::v1_0::*;

use crate::setup::device;
use crate::AppData;

type Vec2 = cgmath::Vector2<f32>;
type Vec3 = cgmath::Vector3<f32>;

pub static VERTICES: [Vertex; 3] = [
    Vertex::new(vec2(0.5, -0.5), vec3(0.0, 0.0, 1.0)),
    Vertex::new(vec2(0.5, 0.5), vec3(0.0, 1.0, 0.0)),
    Vertex::new(vec2(-0.5, 0.5), vec3(1.0, 0.0, 0.0)),
];

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub pos: Vec2,
    pub color: Vec3,
}

impl Vertex {
    const fn new(pos: Vec2, color: Vec3) -> Self {
        Self { pos, color }
    }

    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    pub fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        let pos = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(0)
            .build();

        let color = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(size_of::<Vec2>() as u32)
            .build();

        [pos, color]
    }
}

pub unsafe fn create_vertex_buffer(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    let buffer_info = vk::BufferCreateInfo::builder()
        .size((size_of::<Vertex>() * VERTICES.len()) as u64)
        .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    data.vertext_data.vertex_buffer = device.create_buffer(&buffer_info, None)?;

    let requirements = device.get_buffer_memory_requirements(data.vertext_data.vertex_buffer);

    let memory_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(device::get_memory_type_index(
            instance,
            data,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
            requirements,
        )?);

    data.vertext_data.vertex_buffer_memory = device.allocate_memory(&memory_info, None)?;
    device.bind_buffer_memory(
        data.vertext_data.vertex_buffer,
        data.vertext_data.vertex_buffer_memory,
        0,
    )?;

    let memory = device.map_memory(
        data.vertext_data.vertex_buffer_memory,
        0,
        buffer_info.size,
        vk::MemoryMapFlags::empty(),
    )?;

    memcpy(VERTICES.as_ptr(), memory.cast(), VERTICES.len());
    device.unmap_memory(data.vertext_data.vertex_buffer_memory);

    Ok(())
}
