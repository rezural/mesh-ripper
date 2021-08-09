use std::time::Duration;

use super::inspector::vec_as_dropdown::VecAsDropdown;
use super::resources::actions::{Actions, State as AppState};
use super::resources::asset_load_checker::AssetLoadChecker;
use super::resources::background_meshes::BackgroundMeshes;
use super::resources::camera::CameraSystem;
use super::resources::glob_or_dir_loader::GlobOrDirLoader;
use super::resources::mesh_aabb_estimator::MeshAABBEstimator;
use super::resources::mesh_lookat_estimator::MeshLookAtEstimator;
use super::resources::mesh_pool::MeshPool;
use super::GameState;
use super::{loading::MeshAssets, AppOptions};
use bevy::app::Events;
use bevy::window::WindowFocused;
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
                .with_system(spawn_world.system())
                .with_system(disable_cursor_on_start.system()),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(handle_actions.system())
                .with_system(update_mesh.system())
                .label("update_mesh")
                .with_system(check_lights.system())
                .with_system(check_for_reload.system())
                .with_system(cursor_grab_system.system()),
        )
        .add_system_set(SystemSet::on_exit(GameState::Playing).with_system(remove_player.system()))
        .add_system_set(
            SystemSet::on_update(GameState::Playing).with_system(check_mesh_assets.system()),
        );
    }
}

fn disable_cursor_on_start(
    mut windows: ResMut<Windows>,
    mut query: Query<&mut FlyCamera>,
) {
    let window = windows.get_primary_mut().unwrap();
    for mut camera in query.iter_mut() {
        window.set_cursor_lock_mode(false);
        window.set_cursor_visibility(true);
        camera.enabled = false;
    }
}

fn cursor_grab_system(
    mut windows: ResMut<Windows>,
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
    mut query: Query<&mut FlyCamera>,
    ui_context: Res<EguiContext>,
    focus_events: Res<Events<WindowFocused>>,
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

        let mut focus_lost = false;
        let mut reader = focus_events.get_reader();
        for event in reader.iter(&focus_events) {
            if !event.focused {
                focus_lost = true;
            }
        }

        if key.just_pressed(KeyCode::Escape) || focus_lost {
            window.set_cursor_lock_mode(false);
            window.set_cursor_visibility(true);
            camera.enabled = false;
        }
    }
}

fn check_lights(
    mut commands: Commands,
    actions: Res<Actions>,
    mut state: ResMut<AppState>,
) {
    if actions.spot_lighting {
        if let None = state.spot_lights {
            // state.spot_lights =
            let lights = [
                Vec3::new(200.0, 200.0, 200.0),
                Vec3::new(200.0, 200.0, -200.0),
                Vec3::new(-200.0, 200.0, -200.0),
                Vec3::new(-200.0, 200.0, 200.0),
            ];

            let lights = lights
                .iter()
                .map(|light| {
                    commands
                        .spawn_bundle(LightBundle {
                            transform: Transform::from_translation(*light),
                            light: Light {
                                intensity: actions.lighting_intensity,
                                range: 400_000.0,
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .id()
                })
                .collect();

            state.spot_lights = Some(lights);
        }
    } else {
        if let Some(spot_lights) = &state.spot_lights {
            for &light in spot_lights {
                commands.entity(light).despawn_recursive();
            }
            state.spot_lights = None;
        }
    }
}

fn spawn_camera(
    mut commands: Commands,
    mut ambient_light: ResMut<AmbientLight>,
) {
    ambient_light.color = Color::WHITE;
    ambient_light.brightness = 0.4;

    let fly_camera = FlyCamera {
        max_speed: 1.,
        accel: 2.,

        key_down: KeyCode::Q,
        key_up: KeyCode::E,
        ..Default::default()
    };
    let eye = Vec3::new(0., 40., 20.);
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
    let pool = MeshPool::new(
        fluid_pool_length,
        Duration::from_secs_f32(actions.advance_every),
    );
    commands.insert_resource(pool.clone());
}

fn handle_actions(
    mut commands: Commands,
    mut actions: ResMut<Actions>,
    mut meshes: ResMut<MeshAssets>,
    mut mesh_pool: ResMut<MeshPool>,
    asset_server: Res<AssetServer>,
    mut glob_or_dir_loader: ResMut<GlobOrDirLoader>,
    config: Res<AppOptions>,
    mut camera_system: ResMut<CameraSystem>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    mesh_pool.advance_every = Duration::from_secs_f32(actions.advance_every);
    mesh_pool.frame_direction = actions.frame_direction.clone();
    mesh_pool.paused = actions.paused;

    if actions.reset {
        let material = materials.get_handle(meshes.material.id);
        mesh_pool.reset();
        mesh_pool.redraw(&mut commands, &*meshes, material);
        actions.reset = false;
    }

    mesh_pool.num_fluids = meshes.loaded.len();

    let load_manager = glob_or_dir_loader.load_manager_mut();
    // if the user has chosen a higer asset load lod
    let wanted_lod_len = actions.load_number_of_frames.selected_value();
    if let Some(wanted_lod_len) = wanted_lod_len {
        if wanted_lod_len > (load_manager.loaded.len() + load_manager.loading.len())
            && load_manager.fully_loaded()
        {
            load_manager.next_lod_and_reload(&asset_server);
            meshes.loading = load_manager.loading.clone();
        }
    }

    if actions.initial_lod != load_manager.load_iterator.first_lod {
        load_manager.load_iterator.first_lod = actions.initial_lod;
    }

    // if the user has chosen a different data dir
    if actions.datasets.changed() {
        glob_or_dir_loader.load_manager_mut().clear();
        glob_or_dir_loader.update(
            config.file_glob.clone(),
            actions.datasets.selected_value(),
            &asset_server,
        );
        let load_manager = glob_or_dir_loader.load_manager();
        actions.load_number_of_frames = VecAsDropdown::new(load_manager.load_iterator.get_lods());

        if let Some(dataset) = actions.datasets.selected_value() {
            if let Ok(dir) = std::env::current_dir() {
                let dataset_dir = dir.join(dataset);
                if let Ok(config) = std::fs::read_to_string(dataset_dir.join("mr-config.ron")) {
                    if let Ok(config) = ron::from_str::<Actions>(config.as_str()) {
                        println!("got config");
                        // *actions = config;
                        actions.fluid_color = config.fluid_color;
                        actions.spot_lighting = config.spot_lighting;
                        actions.lighting_intensity = config.lighting_intensity;
                        actions.opacity = config.opacity;
                        actions.material_roughness = config.material_roughness;
                    }
                }
                if let Ok(config) =
                    std::fs::read_to_string(dataset_dir.join("mr-camera-config.ron"))
                {
                    if let Ok(config_camera) = ron::from_str(config.as_str()) {
                        println!("got camera");
                        *camera_system = config_camera;
                    }
                }
            }
        }
    }
}

fn update_mesh(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    fluid_assets: ResMut<MeshAssets>,
    mut actions: ResMut<Actions>,
    mut pool: ResMut<MeshPool>,
    time: Res<Time>,
) {
    // println!("update_mesh: {:?}", time.time_since_startup());`
    let material = materials.get_handle(fluid_assets.material.id);
    let material = materials.get_mut(material.clone());

    if let Some(material) = material {
        material.base_color = actions.fluid_color;
        material.base_color.set_a(actions.opacity);
        material.double_sided = true;
        material.roughness = actions.material_roughness;
        let material = materials.get_handle(fluid_assets.material.id);
        pool.update_fluid(&mut commands, &fluid_assets, material, time.delta());
        if let Some(current_mesh) = pool.current_mesh(&fluid_assets) {
            actions.current_file = current_mesh.0.clone();
            actions.current_frame = pool.current_mesh_index;
        }
    }
}

fn check_for_reload(
    mut actions: ResMut<Actions>,
    config: Res<AppOptions>,
    mut glob_or_dir_loader: ResMut<GlobOrDirLoader>,
    asset_server: Res<AssetServer>,
) {
    if !actions.reload {
        return;
    }

    glob_or_dir_loader.update(
        config.file_glob.clone(),
        actions.datasets.selected_value(),
        &asset_server,
    );

    actions.load_number_of_frames = VecAsDropdown::new_with_selected(
        glob_or_dir_loader.load_manager().load_iterator.get_lods(),
        actions.load_number_of_frames.selected_index(),
    );

    actions.reload = false;
}

fn check_mesh_assets(
    mut commands: Commands,
    time: Res<Time>,
    mut actions: ResMut<Actions>,
    mut fluid_assets: ResMut<MeshAssets>,
    asset_server: Res<AssetServer>,
    mut pool: ResMut<MeshPool>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut glob_or_dir_loader: ResMut<GlobOrDirLoader>,
    mut background_meshes: ResMut<BackgroundMeshes>,
    load_checker: Res<AssetLoadChecker<Mesh>>,
    meshes: Res<Assets<Mesh>>,
    mut query: Query<(&mut FlyCamera, &mut Transform)>,
) {
    load_checker.update(&mut *background_meshes, &*asset_server);
    (*background_meshes).spawn(&mut commands, &mut (*materials));

    let load_manager = glob_or_dir_loader.load_manager_mut();
    load_manager.update_load_state(&asset_server);

    fluid_assets.loaded = load_manager.loaded.clone();
    fluid_assets
        .loaded
        .sort_by(|(a, _), (b, _)| alphanumeric_sort::compare_str(a.as_str(), b.as_str()));

    fluid_assets.loading = load_manager.loading.clone();

    if load_manager.loaded.len() > 0 {
        let material = materials.get_handle(fluid_assets.material.id);
        let material = materials.get_mut(material.clone());

        if let Some(material) = material {
            material.base_color = actions.fluid_color;
            material.base_color.set_a(actions.opacity);
            material.double_sided = true;
            let material = materials.get_handle(fluid_assets.material.id);
            let have_displayed = pool.have_displayed;
            pool.update_fluid(&mut commands, &fluid_assets, material, time.delta());
            if let Some(current_mesh) = pool.current_mesh(&fluid_assets) {
                if !have_displayed {
                    if let Some(mesh) = meshes.get(current_mesh.1.clone()) {
                        if let Ok((_, mut transform)) = query.single_mut() {
                            if let Some(new_transform) = MeshLookAtEstimator::transform(mesh) {
                                (*transform) = new_transform;
                            }
                        }
                    }
                }
                actions.current_file = current_mesh.0.clone();
            }
        }
    }
    // println!(
    //     "fluids_loaded: {}, {}",
    //     fluid_assets.loaded.len(),
    //     load_manager.loaded.len()
    // );
    actions.fluids_loaded = fluid_assets.loaded.len();
    actions.fluids_loaded_percent = (fluid_assets.loaded.len().max(1) as f32
        / (fluid_assets.loaded.len() + load_manager.loading.len()) as f32)
        * 100.;
}

fn remove_player(
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
) {
    for player in player_query.iter() {
        commands.entity(player).despawn();
    }
}
