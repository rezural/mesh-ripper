use crate::support::loader_fu::render::PointRenderOptions;

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
    pub particle_render_style: PointRenderOptions,
    #[inspectable(min = 0.0, max = 5.0, speed = 0.01)]
    pub particle_radius: f32,
    // #[inspectable(min = 100, max = 10000, speed = 10)]
    pub max_particles_render: usize,
    #[inspectable(min = 0.0, max = 1.0, speed = 0.01)]
    pub opacity: f32,
    #[inspectable(label = "# Frames to Initially Load")]
    pub initial_lod: usize,
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
    pub focus_on_mesh: bool,
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
            initial_lod: 100,
            load_number_of_frames: VecAsDropdown::default(),
            datasets: VecAsDropdown::default(),
            current_file: String::from(""),
            show_axis: false,
            spot_lighting: false,
            lighting_intensity: 1000.0,
            material_roughness: 0.089,
            particle_radius: 0.05,
            particle_render_style: PointRenderOptions::Sphere,
            max_particles_render: 1000,
            focus_on_mesh: false,
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
