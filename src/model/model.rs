use std::collections::HashMap;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::BufReader;

use anyhow::Result;
use cgmath::{vec2, vec3};

use crate::vertex::vertex;
use crate::AppData;

impl PartialEq for vertex::Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos && self.color == other.color && self.tex_coord == other.tex_coord
    }
}

impl Eq for vertex::Vertex {}

impl Hash for vertex::Vertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pos[0].to_bits().hash(state);
        self.pos[1].to_bits().hash(state);
        self.pos[2].to_bits().hash(state);
        self.color[0].to_bits().hash(state);
        self.color[1].to_bits().hash(state);
        self.color[2].to_bits().hash(state);
        self.tex_coord[0].to_bits().hash(state);
        self.tex_coord[1].to_bits().hash(state);
    }
}

pub fn load_model(data: &mut AppData) -> Result<()> {
    let mut reader = BufReader::new(File::open(
        "/home/mhl/vulkanalia_tutorial/src/texture/resources/viking_room.obj",
    )?);

    let (models, _) = tobj::load_obj_buf(
        &mut reader,
        &tobj::LoadOptions {
            triangulate: true,
            ..Default::default()
        },
        |_| Ok(Default::default()),
    )?;

    let mut unique_vertices = HashMap::new();

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
                    1.0 - model.mesh.texcoords[tex_coord_offset + 1],
                ),
            };

            if let Some(index) = unique_vertices.get(&vertex) {
                data.vertex_data.indices.push(*index as u32);
            } else {
                let index = data.vertex_data.vertices.len();
                unique_vertices.insert(vertex, index);
                data.vertex_data.vertices.push(vertex);
                data.vertex_data.indices.push(index as u32);
            }
        }
    }

    Ok(())
}
