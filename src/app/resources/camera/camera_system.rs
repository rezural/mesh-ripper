use super::camera_timeline::CameraTimeline;
use crate::app::inspector::vec_as_dropdown::VecAsDropdown;
use bevy::prelude::Transform;
use bevy_inspector_egui::Inspectable;
use serde::*;

#[derive(Inspectable, Debug, Serialize, Deserialize)]
pub struct CameraSystem {
    #[serde(skip)]
    pub record_mode: bool,
    pub show_camera_visualization: bool,
    #[serde(skip)]
    pub follow_camera: bool,
    #[serde(skip)]
    pub current_transform: Transform,
    pub current_timeline: VecAsDropdown<String>,
    pub camera_timelines: Vec<(String, CameraTimeline)>,
}

impl CameraSystem {
    fn enabled_timeline_index(&self) -> Option<usize> {
        if let Some(ci) = self
            .current_timeline
            .selected_value()
            .and_then(|ct| self.camera_timelines.iter().position(|t| t.0 == ct))
        {
            return Some(ci);
        }
        None
    }
    pub fn enabled_timeline(&self) -> Option<&CameraTimeline> {
        if let Some(ct) = self
            .enabled_timeline_index()
            .and_then(|idx| self.camera_timelines.get(idx))
            .and_then(|t| Some(&t.1))
        {
            return Some(ct);
        }
        None
    }

    pub fn enabled_timeline_mut(&mut self) -> Option<&mut CameraTimeline> {
        if let Some(ct) = self
            .enabled_timeline_index()
            .and_then(move |idx| self.camera_timelines.get_mut(idx))
            .and_then(|t| Some(&mut t.1))
        {
            return Some(ct);
        }
        None
    }

    pub fn refresh_current_timeline(&mut self) {
        let from = self
            .camera_timelines
            .iter()
            .map(|(name, _)| name)
            .cloned()
            .collect();

        self.current_timeline.set_from(from);
    }
}

impl Default for CameraSystem {
    fn default() -> Self {
        Self {
            camera_timelines: vec![(String::from("Default"), CameraTimeline::default())],
            current_timeline: VecAsDropdown::new(vec![String::from("Default")]),
            record_mode: false,
            show_camera_visualization: false,
            follow_camera: false,
            current_transform: Transform::identity(),
        }
    }
}
