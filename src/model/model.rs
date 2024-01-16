use std::collections::HashMap;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::BufReader;

use anyhow::Result;
use cgmath::{vec2, vec3};

use crate::vertex::vertex;
use crate::AppData;

pub fn load_model(data: &mut AppData) -> Result<()> {
    // Model

    let mut reader = BufReader::new(File::open("/home/mhl/vulkanalia_tutorial/src/texture/resources/viking_room.obj")?);

    let (models, _) = tobj::load_obj_buf(
        &mut reader,
        &tobj::LoadOptions {
            triangulate: true,
            ..Default::default()
        },
        |_| Ok(Default::default()),
    )?;

    // Vertices / Indices

    for model in &models {
        for index in &model.mesh.indices {
            let pos_offset = (3 * index) as usize;
            let tex_coord_offset = (2 * index) as usize;

            let vertex = vertex::Vertex {
                pos: vec3(
                    model.mesh.positions[pos_offset],
                    model.mesh.positions[pos_offset + 1],
                    model.mesh.positions[pos_offset + 2],
                ),
                color: vec3(1.0, 1.0, 1.0),
                tex_coord: vec2(
                    model.mesh.texcoords[tex_coord_offset],
                    model.mesh.texcoords[tex_coord_offset + 1],
                ),
            };

            data.vertex_data.vertices.push(vertex);
            data.vertex_data
                .indices
                .push(data.vertex_data.indices.len() as u32);
        }
    }

    Ok(())
}
