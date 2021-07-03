use std::path::Path;

use super::resources::load_manager::LoadManager;
use super::resources::mesh_pool::MeshPool;
use super::GameState;
use super::{actions::Actions, loading::MeshAssets, AppOptions};
use bevy::{pbr::AmbientLight, prelude::*, render::camera::PerspectiveProjection};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy_inspector_egui::bevy_egui::EguiContext;

pub struct PlayerPlugin;

pub struct Player;

impl Plugin for PlayerPlugin {
    fn build(
        &self,
        app: &mut AppBuilder,
    ) {
        app.add_plugin(FlyCameraPlugin);

        app.add_system_set(
            SystemSet::on_enter(GameState::Playing)
                .with_system(spawn_camera.system())
                .with_system(spawn_world.system()),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(move_player.system())
                .with_system(check_for_reload.system())
                .with_system(cursor_grab_system.system()),
        )
        .add_system_set(SystemSet::on_exit(GameState::Playing).with_system(remove_player.system()));
    }
}

fn cursor_grab_system(
    mut windows: ResMut<Windows>,
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
    mut query: Query<&mut FlyCamera>,
    ui_context: Res<EguiContext>,
) {
    let window = windows.get_primary_mut().unwrap();
    for mut camera in query.iter_mut() {
        if btn.just_pressed(MouseButton::Left) {
            if !ui_context.ctx().wants_pointer_input() {
                window.set_cursor_lock_mode(true);
                window.set_cursor_visibility(false);
                camera.enabled = true;
            }
        }

        if key.just_pressed(KeyCode::Escape) {
            window.set_cursor_lock_mode(false);
            window.set_cursor_visibility(true);
            camera.enabled = false;
        }
    }
}

fn spawn_camera(
    mut commands: Commands,
    mut ambient_light: ResMut<AmbientLight>,
) {
    ambient_light.color = Color::WHITE;
    ambient_light.brightness = 0.8;

    // let lights = [
    //     Vec3::new(200.0, 200.0, 200.0),
    //     Vec3::new(200.0, 200.0, -200.0),
    //     Vec3::new(-200.0, 200.0, -200.0),
    //     Vec3::new(-200.0, 200.0, 200.0),
    // ];

    // for light in lights.iter() {
    //     commands.spawn_bundle(LightBundle {
    //         transform: Transform::from_translation(*light),
    //         light: Light {
    //             intensity: 10_000.0,
    //             range: 3_000_000.0,
    //             ..Default::default()
    //         },
    //         ..Default::default()
    //     });
    // }

    let fly_camera = FlyCamera {
        max_speed: 2.,
        accel: 8.,

        key_down: KeyCode::Q,
        key_up: KeyCode::E,
        ..Default::default()
    };
    let eye = Vec3::new(0., 20., 20.);
    let target = Vec3::new(0., 0., 0.);
    commands
        .spawn()
        .insert_bundle(PerspectiveCameraBundle {
            transform: Transform::from_translation(eye).looking_at(target, Vec3::Y),
            perspective_projection: PerspectiveProjection {
                fov: std::f32::consts::PI / 5.0,
                near: 0.05,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(fly_camera);
}

fn spawn_world(
    mut commands: Commands,
    fluid_assets: Res<MeshAssets>,
    actions: Res<Actions>,
) {
    let fluid_pool_length = fluid_assets.loaded.len();
    let pool = MeshPool::new(fluid_pool_length, actions.advance_every);
    commands.insert_resource(pool.clone());
}

//FIXME move all this out into stepper.rs, or something
fn move_player(
    commands: Commands,
    time: Res<Time>,
    mut actions: ResMut<Actions>,
    mut fluid_assets: ResMut<MeshAssets>,
    mut pool: ResMut<MeshPool>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut load_manager: ResMut<LoadManager>,
) {
    pool.advance_every = actions.advance_every;
    pool.frame_direction = actions.frame_direction.clone();
    pool.paused = actions.paused;

    if actions.reset {
        pool.reset();
        actions.reset = false;
    }
    pool.num_fluids = fluid_assets.loaded.len();

    // if the user has chosen a higer asset load lod
    let wanted_lod_len = actions.load_number_of_frames.selected_value();
    if wanted_lod_len > (load_manager.loaded.len() + load_manager.loading.len())
        && load_manager.fully_loaded()
    {
        load_manager.next_lod_and_reload(&asset_server);
        fluid_assets.loading = load_manager.loading.clone();
    }

    let material = materials.get_handle(fluid_assets.material.id);
    let material = materials.get_mut(material.clone());

    if let Some(material) = material {
        material.base_color = actions.fluid_color;
        material.base_color.set_a(actions.opacity);
        material.double_sided = true;
        let material = materials.get_handle(fluid_assets.material.id);
        pool.update_fluid(commands, (*fluid_assets).clone(), material, time.delta());
    }
}

// FIXME: use LoadManager
fn check_for_reload(
    mut actions: ResMut<Actions>,
    fluid_assets: ResMut<MeshAssets>,
    config: Res<AppOptions>,
    asset_server: ResMut<AssetServer>,
) {
    if !actions.reload {
        return;
    }

    add_extra_files_to_load(config, fluid_assets, asset_server);
    actions.reload = false;
}

// FIXME: use LoadManager
fn add_extra_files_to_load(
    config: Res<AppOptions>,
    mut fluid_assets: ResMut<MeshAssets>,
    asset_server: ResMut<AssetServer>,
) {
    let glob = config.file_glob.as_str();

    let fluid_files: Vec<String> = glob::glob(glob)
        .expect("Loading fluid from assets failed in glob")
        .map(|entry| entry.unwrap().to_string_lossy().to_string())
        .collect();

    let not_already_loading = fluid_files.iter().filter(|&file_name| {
        !fluid_assets
            .loading
            .iter()
            .any(|(loaded_name, _)| file_name == loaded_name)
    });

    let fluids_to_load = not_already_loading.filter(|&file_name| {
        !fluid_assets
            .loaded
            .iter()
            .any(|(loaded_name, _)| file_name == loaded_name)
    });

    let fluids_to_load: Vec<(String, HandleUntyped)> = fluids_to_load
        .map(|fluid_file| {
            (
                fluid_file.clone(),
                asset_server.load_untyped(Path::new(&fluid_file).strip_prefix("assets/").unwrap()),
            )
        })
        .collect();

    fluid_assets.loading.extend(fluids_to_load);
}

fn remove_player(
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
) {
    for player in player_query.iter() {
        commands.entity(player).despawn();
    }
}
