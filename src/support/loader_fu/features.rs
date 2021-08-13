use bevy::{prelude::Mesh, render::mesh::Indices};

pub struct Features<'a> {
    mesh: &'a Mesh,
}

impl<'a> Features<'a> {
    pub fn new(mesh: &'a Mesh) -> Self {
        Self { mesh }
    }

    pub fn has_vertices(&self) -> bool {
        self.vertices().is_some()
    }

    pub fn has_indices(&self) -> bool {
        self.indices().is_some()
    }

    pub fn has_normals(&self) -> bool {
        self.normals().is_some()
    }

    pub fn has_velocities(&self) -> bool {
        self.velocities().is_some()
    }

    pub fn vertices(&self) -> Option<&Vec<[f32; 3]>> {
        let vertices = self.mesh.attribute(Mesh::ATTRIBUTE_POSITION);
        let vertices = if let Some(vertices) = vertices {
            match vertices {
                bevy::render::mesh::VertexAttributeValues::Float3(vertices) => Some(vertices),
                _ => None,
            }
        } else {
            None
        };

        vertices
    }

    pub fn normals(&self) -> Option<&Vec<[f32; 3]>> {
        let normals = self.mesh.attribute(Mesh::ATTRIBUTE_NORMAL);
        let normals = if let Some(normals) = normals {
            match normals {
                bevy::render::mesh::VertexAttributeValues::Float3(normals) => Some(normals),
                _ => None,
            }
        } else {
            None
        };

        normals
    }

    pub fn indices(&self) -> Option<&Indices> {
        self.mesh.indices()
    }

    pub fn velocities(&self) -> Option<&Vec<[f32; 3]>> {
        self.normals()
    }
}
