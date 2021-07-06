use super::inspector::vec_as_dropdown::VecAsDropdown;
use super::resources::glob_or_dir_loader::GlobOrDirLoader;
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
        .add_system_set(SystemSet::on_exit(GameState::Playing).with_system(remove_player.system()))
        .add_system_set(
            SystemSet::on_update(GameState::Playing).with_system(check_mesh_assets.system()),
        );
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
    ambient_light.brightness = 100.;

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
        max_speed: 1.,
        accel: 8.,

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
    mut glob_or_dir_loader: ResMut<GlobOrDirLoader>,
    config: Res<AppOptions>,
) {
    pool.advance_every = actions.advance_every;
    pool.frame_direction = actions.frame_direction.clone();
    pool.paused = actions.paused;

    if actions.reset {
        pool.reset();
        actions.reset = false;
    }
    pool.num_fluids = fluid_assets.loaded.len();

    let load_manager = glob_or_dir_loader.load_manager_mut();
    // if the user has chosen a higer asset load lod
    let wanted_lod_len = actions.load_number_of_frames.selected_value();
    if let Some(wanted_lod_len) = wanted_lod_len {
        if wanted_lod_len > (load_manager.loaded.len() + load_manager.loading.len())
            && load_manager.fully_loaded()
        {
            load_manager.next_lod_and_reload(&asset_server);
            fluid_assets.loading = load_manager.loading.clone();
        }
    }

    // if the user has chosen a different data dir
    if actions.datasets.changed() {
        println!("changed: {:?}", actions.datasets.selected_value());

        glob_or_dir_loader.update(
            config.file_glob.clone(),
            actions.datasets.selected_value(),
            &asset_server,
        )
    }

    let material = materials.get_handle(fluid_assets.material.id);
    let material = materials.get_mut(material.clone());

    if let Some(material) = material {
        material.base_color = actions.fluid_color;
        material.base_color.set_a(actions.opacity);
        material.double_sided = true;
        let material = materials.get_handle(fluid_assets.material.id);
        pool.update_fluid(commands, &fluid_assets, material, time.delta());
        if let Some(current_mesh) = pool.current_mesh(&fluid_assets) {
            actions.current_file = current_mesh.0.clone();
        }
    }
}

fn check_for_reload(
    mut actions: ResMut<Actions>,
    config: Res<AppOptions>,
    mut glob_or_dir_loader: ResMut<GlobOrDirLoader>,
) {
    if !actions.reload {
        return;
    }

    if let Some(glob) = config.file_glob.clone() {
        let new_assets: Vec<String> = glob::glob(glob.as_str())
            .expect("Loading fluid from assets failed in glob")
            .map(|entry| entry.unwrap().to_string_lossy().to_string())
            .collect();

        let load_manager = glob_or_dir_loader.load_manager_mut();

        load_manager.add_new_assets(new_assets);
        actions.load_number_of_frames = VecAsDropdown::new_with_selected(
            load_manager.load_iterator.get_lods(),
            actions.load_number_of_frames.selected_index(),
        );
    }

    actions.reload = false;
}

fn check_mesh_assets(
    commands: Commands,
    time: Res<Time>,
    mut actions: ResMut<Actions>,
    mut fluid_assets: ResMut<MeshAssets>,
    asset_server: Res<AssetServer>,
    mut pool: ResMut<MeshPool>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut glob_or_dir_loader: ResMut<GlobOrDirLoader>,
) {
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
            pool.update_fluid(commands, &fluid_assets, material, time.delta());
            if let Some(current_mesh) = pool.current_mesh(&fluid_assets) {
                actions.current_file = current_mesh.0.clone();
            }
        }
    }
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
