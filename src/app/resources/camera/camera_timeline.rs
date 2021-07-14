use bevy::prelude::Transform;
use bevy_inspector_egui::Inspectable;
use nalgebra::{Isometry3, Translation3, UnitQuaternion};
use serde::*;

use crate::app::Real;
// use parry3d::math::{Isometry, Real};

#[derive(Default, Clone, Debug, Inspectable, Serialize, Deserialize)]
pub struct CameraTimeline {
    pub timeline: Vec<CameraFrame>,
}

type MintTransform = (mint::Vector3<Real>, mint::Quaternion<Real>);

impl CameraTimeline {
    pub fn add_frame(
        &mut self,
        frame: usize,
        pose: Transform,
    ) {
        self.timeline.push(CameraFrame::from_transform(frame, pose));
        self.timeline.sort_by(|a, b| a.frame.cmp(&b.frame))
    }

    // FIXME: unit test this code, it is slightly tricky / prone to failure possibly
    pub fn transform_at_frame(
        &self,
        frame: usize,
    ) -> Option<Isometry3<Real>> {
        if self.timeline.len() < 1 {
            return None;
        }
        let available_frames: Vec<usize> = self.timeline.iter().map(|f| f.frame).collect();
        let exact_frame = available_frames.contains(&frame);
        if exact_frame {
            if let Some(idx) = available_frames.iter().position(|e| *e == frame) {
                return Some(self.timeline.get(idx).unwrap().pose);
            }
        } else {
            let mut available_frames_idx = self
                .timeline
                .iter()
                .enumerate()
                .map(|(idx, f)| (idx, f.frame));
            let candidate_frame = available_frames_idx.find(|f| f.1 > frame);
            if let Some(candidate_frame) = candidate_frame {
                if candidate_frame.0 > 0 {
                    if candidate_frame.0 < self.timeline.len() {
                        let (before, after) = (
                            self.timeline[candidate_frame.0 - 1].clone(),
                            self.timeline[candidate_frame.0].clone(),
                        );
                        return Some(Self::lerp_frames(&before, &after, frame));
                    } else {
                        return Some(self.timeline.last().unwrap().pose);
                    }
                } else {
                    return Some(self.timeline.first().unwrap().pose);
                }
            }
        }

        None
    }

    pub fn lerp_frames(
        a: &CameraFrame,
        b: &CameraFrame,
        frame: usize,
    ) -> Isometry3<Real> {
        let t = ((frame - a.frame) as f32) / ((b.frame - a.frame) as f32);

        let from = a.pose;
        let to = b.pose;

        from.lerp_slerp(&to, t) // lerp_slerp!
    }
}
#[derive(Clone, Debug, Inspectable, Serialize, Deserialize)]
pub struct CameraFrame {
    pub frame: usize,
    #[inspectable(ignore)]
    pub pose: Isometry3<Real>,
}

impl CameraFrame {
    pub fn from_transform(
        frame: usize,
        pose: Transform,
    ) -> Self {
        Self {
            frame,
            pose: Self::transform_to_isometry(pose),
        }
    }

    fn transform_to_isometry(transform: Transform) -> Isometry3<Real> {
        let from: MintTransform = (transform.translation.into(), transform.rotation.into());
        Isometry3::from_parts(
            Translation3::new(from.0.x, from.0.y, from.0.z),
            UnitQuaternion::from_quaternion(from.1.into()),
        )
    }

    pub fn isometry_to_transform(isometry: Isometry3<Real>) -> Transform {
        let transform: MintTransform =
            (isometry.translation.vector.into(), isometry.rotation.into());

        Transform {
            translation: transform.0.into(),
            rotation: transform.1.into(),
            ..Default::default()
        }
    }
}

impl Default for CameraFrame {
    fn default() -> Self {
        Self {
            frame: 0,
            pose: Isometry3::identity(),
        }
    }
}
