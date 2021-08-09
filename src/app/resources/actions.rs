use super::{
    super::inspector::vec_as_dropdown::VecAsDropdown, background_meshes::BackgroundMeshes,
};
use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use serde::*;

#[derive(Inspectable, Debug, Serialize, Deserialize)]
pub struct Actions {
    pub current_frame: usize,
    pub frame_direction: FrameDirection,
    #[inspectable(min = 0.0, max = 1.0, speed = 0.01)]
    pub advance_every: f32,
    pub reset: bool,
    pub paused: bool,
    pub fluids_loaded: usize,
    pub fluids_loaded_percent: f32,
    pub reload: bool,
    pub fluid_color: Color,
    #[inspectable(min = 0.0, max = 1.0, speed = 0.01)]
    pub opacity: f32,
    #[inspectable(label = "# of Frames to Load")]
    #[serde(skip)]
    pub load_number_of_frames: VecAsDropdown<usize>,
    #[inspectable(label = "Load from Dataset")]
    #[serde(skip)]
    pub datasets: VecAsDropdown<String>,
    pub current_file: String,
    pub show_axis: bool,
    pub spot_lighting: bool,
    #[inspectable(min = 0.0, max = 10_000_000.0, speed = 100.)]
    pub lighting_intensity: f32,
    #[inspectable(min = 0.0, max = 1.0, speed = 0.01)]
    pub material_roughness: f32,
    pub camera_mode: bool,
}

impl Default for Actions {
    fn default() -> Self {
        Self {
            current_frame: 0,
            advance_every: 0.1,
            // last_time_drawn: Instant::now(),
            paused: true,
            reset: false,
            frame_direction: Default::default(),
            fluids_loaded: 0,
            fluids_loaded_percent: 0.,
            reload: false,
            fluid_color: Color::rgb(95. / 255., 133. / 255., 194. / 255.),
            opacity: 1.0,
            load_number_of_frames: VecAsDropdown::default(),
            datasets: VecAsDropdown::default(),
            current_file: String::from(""),
            show_axis: false,
            spot_lighting: false,
            lighting_intensity: 1000.0,
            material_roughness: 0.089,
            camera_mode: false,
        }
    }
}

#[derive(Inspectable, Debug, Clone, Serialize, Deserialize)]
pub enum FrameDirection {
    Forward,
    Back,
}

impl Default for FrameDirection {
    fn default() -> Self {
        FrameDirection::Forward
    }
}

pub struct State {
    pub spot_lights: Option<Vec<Entity>>,
    pub background_meshes: BackgroundMeshes,
}

impl Default for State {
    fn default() -> Self {
        let spot_lights = Some(Vec::new());
        Self {
            spot_lights,
            background_meshes: Default::default(),
        }
    }
}
