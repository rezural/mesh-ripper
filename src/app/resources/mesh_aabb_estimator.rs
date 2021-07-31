use bevy::prelude::*;
use parry3d::{bounding_volume::AABB, math::*};

pub struct MeshAABBEstimator {}

impl MeshAABBEstimator {
    pub fn aabb(mesh: &Mesh) -> Option<AABB> {
        let vertices = mesh.attribute("Vertex_Position");
        if let Some(vertices) = vertices {
            if let Some(vertices) = match vertices {
                bevy::render::mesh::VertexAttributeValues::Float3(vertices) => Some(vertices),
                _ => None,
            } {
                let first = vertices[0];
                let mut aabb = AABB::from_half_extents(
                    Point::new(first[0], first[1], first[2]),
                    Vector::new(0., 0., 0.),
                );
                for vertex in vertices {
                    aabb.take_point(Point::new(vertex[0], vertex[1], vertex[2]))
                }
                return Some(aabb);
            }
        }
        None
    }

    pub fn pose_from_aabb(aabb: &AABB) -> Point<Real> {
        Point::new(
            aabb.center().coords.x,
            aabb.maxs.coords.y * 1.5,
            aabb.maxs.coords.z * 1.5,
        )
    }
}
