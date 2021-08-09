use bevy::prelude::{Mesh, Transform};
use glam::Vec3;

use super::mesh_aabb_estimator::MeshAABBEstimator;

pub struct MeshLookAtEstimator {}

impl MeshLookAtEstimator {
    pub fn transform(mesh: &Mesh) -> Option<Transform> {
        if let Some(aabb) = MeshAABBEstimator::aabb(mesh) {
            let eye = MeshAABBEstimator::pose_from_aabb(&aabb);
            let target = aabb.center();

            return Some(
                Transform::from_translation(Vec3::new(eye.x, eye.y, eye.z))
                    .looking_at(Vec3::new(target.x, target.y, target.z), Vec3::Y),
            );
        }
        None
    }
}
