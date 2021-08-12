use bevy::prelude::{Mesh, Transform};
use glam::Vec3;
use parry3d::bounding_volume::AABB;

use super::mesh_aabb_estimator::MeshAABBEstimator;

pub struct MeshLookAtEstimator {}

impl MeshLookAtEstimator {
    pub fn transform(mesh: &Mesh) -> Option<Transform> {
        if let Some(aabb) = MeshAABBEstimator::aabb(mesh) {
            let eye = Self::eye(&aabb);
            let target = Self::target(&aabb);

            return Some(Transform::from_translation(eye).looking_at(target, Vec3::Y));
        }
        None
    }

    pub fn eye_target(mesh: &Mesh) -> Option<(Vec3, Vec3)> {
        if let Some(aabb) = MeshAABBEstimator::aabb(mesh) {
            return Some((Self::eye(&aabb), Self::target(&aabb)));
        }
        None
    }

    pub fn eye(aabb: &AABB) -> Vec3 {
        let pose = MeshAABBEstimator::pose_from_aabb(&aabb);
        return Vec3::new(pose.x, pose.y, pose.z);
    }

    pub fn target(aabb: &AABB) -> Vec3 {
        let target = aabb.center();
        Vec3::new(target.x, target.y, target.z)
    }
}
