use bevy::prelude::Transform;
use bevy_inspector_egui::Inspectable;
use nalgebra::{Isometry3, Translation3, UnitQuaternion};

// use parry3d::math::{Isometry, Real};

#[derive(Default, Clone, Debug, Inspectable)]
pub struct CameraTimeline {
    pub timeline: Vec<CameraFrame>,
}

impl CameraTimeline {
    pub fn add_frame(
        &mut self,
        frame: usize,
        pose: Transform,
    ) {
        self.timeline.push(CameraFrame::new(frame, pose));
        self.timeline.sort_by(|a, b| a.frame.cmp(&b.frame))
    }

    // FIXME: unit test this code, it is slightly tricky / prone to failure possibly
    pub fn transform_at_frame(
        &self,
        frame: usize,
    ) -> Option<Transform> {
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
    ) -> Transform {
        let t = ((frame - a.frame) as f32) / ((b.frame - a.frame) as f32);

        let from = a.pose;
        let to = b.pose;
        type Real = f32;

        type MintTransform = (mint::Vector3<Real>, mint::Quaternion<Real>);

        let fromm: MintTransform = (from.translation.into(), from.rotation.into());
        let tom: MintTransform = (to.translation.into(), to.rotation.into());

        let fromna = Isometry3::from_parts(
            Translation3::new(fromm.0.x, fromm.0.y, fromm.0.z),
            UnitQuaternion::from_quaternion(fromm.1.into()),
        );

        let tona = Isometry3::from_parts(
            Translation3::new(tom.0.x, tom.0.y, tom.0.z),
            UnitQuaternion::from_quaternion(tom.1.into()),
        );

        let lerp = fromna.lerp_slerp(&tona, t);
        let lerp: MintTransform = (lerp.translation.vector.into(), lerp.rotation.into());
        Transform {
            translation: lerp.0.into(),
            rotation: lerp.1.into(),
            ..Default::default()
        }
    }
}
#[derive(Clone, Debug, Inspectable)]
pub struct CameraFrame {
    pub frame: usize,
    pub pose: Transform,
}

impl CameraFrame {
    pub fn new(
        frame: usize,
        pose: Transform,
    ) -> Self {
        Self { frame, pose }
    }
}

impl Default for CameraFrame {
    fn default() -> Self {
        Self {
            frame: 0,
            pose: Transform::identity(),
        }
    }
}
