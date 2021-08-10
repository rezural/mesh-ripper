use super::loading::MeshAssets;
use super::resources::actions::{Actions, FrameDirection, State as AppState};
use super::resources::camera::CameraSystem;
use super::resources::mesh_pool::MeshPool;
use super::GameState;
use bevy::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(
        &self,
        app: &mut AppBuilder,
    ) {
        app.add_system_set(
            SystemSet::on_update(GameState::Playing).with_system(set_movement_actions.system()),
        );
        app.init_resource::<Actions>().init_resource::<AppState>();
    }
}
fn set_movement_actions(
    mut commands: Commands,
    mut actions: ResMut<Actions>,
    keyboard_input: Res<Input<KeyCode>>,
    mut mesh_pool: ResMut<MeshPool>,
    fluid_assets: ResMut<MeshAssets>,
    materials: ResMut<Assets<StandardMaterial>>,
    mut camera_system: ResMut<CameraSystem>,
) {
    if keyboard_input.just_pressed(KeyCode::T) {
        actions.frame_direction = FrameDirection::Forward;
    }

    if keyboard_input.just_pressed(KeyCode::B) {
        actions.frame_direction = FrameDirection::Back;
    }

    if keyboard_input.just_pressed(KeyCode::X) {
        actions.paused = !actions.paused;
    }

    if actions.paused {
        let material = materials.get_handle(fluid_assets.material.id);
        if keyboard_input.pressed(KeyCode::Left) {
            mesh_pool.despawn_mesh(&mut commands);
            mesh_pool.retreat();
            mesh_pool.spawn_mesh(&*fluid_assets, material.clone(), &mut commands)
        }

        if keyboard_input.pressed(KeyCode::Right) {
            mesh_pool.despawn_mesh(&mut commands);
            mesh_pool.advance();
            mesh_pool.spawn_mesh(&*fluid_assets, material.clone(), &mut commands)
        }
    }

    if keyboard_input.just_pressed(KeyCode::R) {
        if keyboard_input.pressed(KeyCode::LControl) {
            actions.reload = true;
        } else {
            actions.reset = true;
        }
    }

    if keyboard_input.just_pressed(KeyCode::F) {
        if keyboard_input.pressed(KeyCode::LControl) {
            actions.focus_on_mesh = true;
        } else {
            if actions.advance_every > 0.019 {
                actions.advance_every -= 0.01;
            } else {
                actions.advance_every -= 0.001;
            }
        }
    }

    if keyboard_input.just_pressed(KeyCode::G) {
        if actions.advance_every > 0.019 {
            actions.advance_every += 0.01;
        } else {
            actions.advance_every += 0.001;
        }
    }

    if keyboard_input.just_pressed(KeyCode::C) {
        if keyboard_input.pressed(KeyCode::LControl) {
            camera_system.follow_camera = !camera_system.follow_camera;
        }
    }
}
