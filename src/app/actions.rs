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
        app.init_resource::<Actions>().add_system_set(
            SystemSet::on_update(GameState::Playing).with_system(set_movement_actions.system()),
        );
    }
}

#[derive(Inspectable, Debug, Clone)]
pub enum FrameDirection {
    Paused,
    Forward,
    Back,
}

impl Default for FrameDirection {
    fn default() -> Self {
        FrameDirection::Paused
    }
}

#[derive(Inspectable)]
pub struct Actions {
    pub frame_direction: FrameDirection,
    pub advance_every: Duration,
    pub reset: bool,
    pub fluids_loaded: usize,
    pub fluids_loaded_percent: f32,
    pub reload: bool,
    pub fluid_color: Color,
    #[inspectable(min = 0.0, max = 1.0, speed = 0.01)] // attributes get passed to each child
    pub opacity: f32,
    pub lods: VecAsDropdown<usize>,
}

impl Default for Actions {
    fn default() -> Self {
        Self {
            advance_every: Duration::from_secs_f32(1. / 10.),
            // last_time_drawn: Instant::now(),
            reset: false,
            frame_direction: Default::default(),
            fluids_loaded: 0,
            fluids_loaded_percent: 0.,
            reload: false,
            fluid_color: Color::rgb(95. / 255., 133. / 255., 194. / 255.),
            opacity: 0.96,
            lods: VecAsDropdown::default(),
        }
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
        actions.frame_direction = FrameDirection::Paused;
    }

    if keyboard_input.just_pressed(KeyCode::R) && !keyboard_input.pressed(KeyCode::LControl) {
        actions.reset = true;
    }

    if keyboard_input.just_pressed(KeyCode::R) && keyboard_input.pressed(KeyCode::LControl) {
        actions.reload = true;
    }

    if keyboard_input.just_pressed(KeyCode::F) {
        if actions.advance_every.as_millis() > 19 {
            actions.advance_every -= Duration::from_millis(10);
        } else if actions.advance_every.as_millis() > 1 {
            actions.advance_every -= Duration::from_millis(1);
        }
    }

    if keyboard_input.just_pressed(KeyCode::G) {
        if actions.advance_every.as_millis() > 9 {
            actions.advance_every += Duration::from_millis(10);
        } else {
            actions.advance_every += Duration::from_millis(1);
        }
    }
}
