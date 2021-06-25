use std::path::Path;

use super::resources::load_manager::LoadManager;
use super::resources::mesh_pool::MeshPool;
use super::GameState;
use super::{actions::Actions, loading::MeshAssets, AppOptions};
use bevy::{pbr::AmbientLight, prelude::*, render::camera::PerspectiveProjection};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};

use super::actions::State as AppState;
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
                .with_system(check_for_reload.system()),
        )
        .add_system_set(SystemSet::on_exit(GameState::Playing).with_system(remove_player.system()));
    }
}

fn spawn_camera(
    mut commands: Commands,
    mut ambient_light: ResMut<AmbientLight>,
) {
    ambient_light.color = Color::WHITE;
    ambient_light.brightness = 0.4;
    let fly_camera = FlyCamera {
        max_speed: 4.,
        accel: 4.,
        key_down: KeyCode::Q,
        key_up: KeyCode::E,
        ..Default::default()
    };
    let eye = Vec3::new(0., 20., 20.);
    let target = Vec3::new(0., 0., 0.);
    commands
        .spawn_bundle(PerspectiveCameraBundle {
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
    materials: ResMut<Assets<StandardMaterial>>,
    fluid_assets: Res<MeshAssets>,
    actions: Res<Actions>,
    time: Res<Time>,
) {
    let fluid_pool_length = fluid_assets.loaded.len();
    let mut pool = MeshPool::new(fluid_pool_length, actions.advance_every);
    commands.insert_resource(pool.clone());

    let water_material = materials.get_handle(fluid_assets.material.id);
    pool.update_fluid(
        commands,
        (*fluid_assets).clone(),
        water_material,
        time.delta(),
    );
}

//FIXMEL move all this out into stepper.rs, or something
fn move_player(
    commands: Commands,
    time: Res<Time>,
    mut actions: ResMut<Actions>,
    // mut state: ResMut<AppState>,
    mut fluid_assets: ResMut<MeshAssets>,
    mut pool: ResMut<MeshPool>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut load_manager: ResMut<LoadManager>,
) {
    pool.advance_every = actions.advance_every;
    pool.frame_direction = actions.frame_direction.clone();

    if actions.reset {
        pool.reset();
        actions.reset = false;
    }
    pool.num_fluids = fluid_assets.loaded.len();

    // if the user has chosen a higer asset load lod
    // let wanted_lod_len = actions.lods.selected_value();
    // if wanted_lod_len > (load_manager.loaded.len() + load_manager.loading.len()) {
    //     load_manager.next_lod_and_reload(&asset_server);
    //     state.load_iterator = load_manager.load_iterator.clone();
    //     fluid_assets.loading = load_manager.loading.clone();
    // }

    let material = materials.get_handle(fluid_assets.material.id);
    let material = materials.get_mut(material.clone());

    if let Some(material) = material {
        material.base_color = actions.fluid_color;
        let material = materials.get_handle(fluid_assets.material.id);
        pool.update_fluid(commands, (*fluid_assets).clone(), material, time.delta());
    }
}

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
