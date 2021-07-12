use std::time::Duration;

use super::inspector::vec_as_dropdown::VecAsDropdown;
use super::GameState;
use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;

pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(
        &self,
        app: &mut AppBuilder,
    ) {
        app.add_system_set(
            SystemSet::on_update(GameState::Playing).with_system(set_movement_actions.system()),
        );
        app.init_resource::<Actions>().init_resource::<State>();
    }
}

#[derive(Inspectable, Debug, Clone)]
pub enum FrameDirection {
    Forward,
    Back,
}

impl Default for FrameDirection {
    fn default() -> Self {
        FrameDirection::Forward
    }
}

#[derive(Inspectable, Debug)]
pub struct Actions {
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
    pub load_number_of_frames: VecAsDropdown<usize>,
    #[inspectable(label = "Load from Dataset")]
    pub datasets: VecAsDropdown<String>,
    pub current_file: String,
    pub show_axis: bool,
    pub spot_lighting: bool,
    #[inspectable(min = 0.0, max = 10_000_000.0, speed = 100.)]
    pub lighting_intensity: f32,
    #[inspectable(min = 0.0, max = 1.0, speed = 0.01)]
    pub material_roughness: f32,
}

impl Default for Actions {
    fn default() -> Self {
        Self {
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
        }
    }
}

pub struct State {
    pub spot_lights: Option<Vec<Entity>>,
}

impl Default for State {
    fn default() -> Self {
        let spot_lights = Some(Vec::new());
        Self { spot_lights }
    }
}

fn set_movement_actions(
    mut actions: ResMut<Actions>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::T) {
        actions.frame_direction = FrameDirection::Forward;
    }

    if keyboard_input.just_pressed(KeyCode::B) {
        actions.frame_direction = FrameDirection::Back;
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        actions.paused = !actions.paused;
    }

    if keyboard_input.just_pressed(KeyCode::R) && !keyboard_input.pressed(KeyCode::LControl) {
        actions.reset = true;
    }

    if keyboard_input.just_pressed(KeyCode::R) && keyboard_input.pressed(KeyCode::LControl) {
        actions.reload = true;
    }

    if keyboard_input.just_pressed(KeyCode::F) {
        if actions.advance_every > 0.019 {
            actions.advance_every -= 0.1;
        } else if actions.advance_every > 1.0 {
            actions.advance_every -= 0.01;
        }
    }

    if keyboard_input.just_pressed(KeyCode::G) {
        if actions.advance_every > 0.09 {
            actions.advance_every += 0.1;
        } else {
            actions.advance_every += 0.01;
        }
    }
}
