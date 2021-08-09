use super::loading::MeshAssets;
use super::resources::actions::{Actions, State as AppState};
use super::resources::camera::*;
use super::resources::glob_or_dir_loader::GlobOrDirLoader;
use super::resources::mesh_lookat_estimator::MeshLookAtEstimator;
use super::resources::mesh_pool::MeshPool;
use super::GameState;
use bevy::prelude::*;
use bevy_fly_camera::FlyCamera;
pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(
        &self,
        app: &mut AppBuilder,
    ) {
        app.add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(camera_timeline_system.system())
                .with_system(state_plumbing.system())
                .after("update_mesh"),
        );
        app.init_resource::<Actions>().init_resource::<AppState>();
    }
}

fn camera_timeline_system(
    mut commands: Commands,
    materials: ResMut<Assets<StandardMaterial>>,
    meshes: ResMut<Assets<Mesh>>,
    mut camera_system: ResMut<CameraSystem>,
    mut visualization: ResMut<CameraSystemVisualization>,
    mut query: Query<(&mut FlyCamera, &mut Transform)>,
    keyboard_input: Res<Input<KeyCode>>,
    mut loader: ResMut<GlobOrDirLoader>,
    pool: ResMut<MeshPool>,
    asset_server: Res<AssetServer>,
    mut actions: ResMut<Actions>,
    fluid_assets: ResMut<MeshAssets>,
    // time: Res<Time>,
) {
    // println!("camera-timeline-system: {:?}", time.time_since_startup());
    if camera_system.record_mode {
        // we need the highest LOD
        let load_manager = (&mut *loader).load_manager_mut();
        load_manager.reload(&*asset_server);
        let load_iterator = &loader.load_manager().load_iterator;

        if let Some(timeline) = camera_system.enabled_timeline_mut() {
            if keyboard_input.just_pressed(KeyCode::C) && !keyboard_input.pressed(KeyCode::LControl)
            {
                if let Ok((_, transform)) = query.single_mut() {
                    let camera_pose = transform;
                    timeline.add_frame(
                        load_iterator.full_index_from_lod_index(pool.current_mesh_index),
                        *camera_pose,
                    );
                }
            }
        }
    }
    let load_iterator = &loader.load_manager().load_iterator;

    // fixme move to config_save_system
    if keyboard_input.pressed(KeyCode::LControl) {
        if keyboard_input.just_pressed(KeyCode::S) {
            if let Some(data_dir) = actions.datasets.selected_value() {
                if let Ok(root) = std::env::current_dir() {
                    let dir_path = root.join(data_dir);
                    println!("dir_path: {}", dir_path.to_string_lossy());
                    if let Ok(config) = ron::ser::to_string_pretty(&*actions, Default::default()) {
                        match std::fs::write(dir_path.join("mr-config.ron"), config) {
                            Ok(_) => {
                                println!("Saved config Successfully")
                            }
                            Err(e) => println!("Couldn't save config: {:?}", e),
                        }
                    }
                    if let Ok(camera_config) =
                        ron::ser::to_string_pretty(&*camera_system, Default::default())
                    {
                        match std::fs::write(dir_path.join("mr-camera-config.ron"), camera_config) {
                            Ok(_) => {
                                println!("Saved camera config Successfully")
                            }
                            Err(e) => println!("Couldn't save camera config: {:?}", e),
                        }
                    }
                }
            }
        }
    }

    if actions.focus_on_mesh {
        if let Some(current_mesh) = pool.current_mesh(&fluid_assets) {
            if let Some(mesh) = meshes.get(current_mesh.1.clone()) {
                if let Ok((_, mut transform)) = query.single_mut() {
                    if let Some(new_transform) = MeshLookAtEstimator::transform(mesh) {
                        (*transform) = new_transform;
                    }
                }
            }
        }
        actions.focus_on_mesh = false;
    }

    if camera_system.show_camera_visualization {
        //FIXME: we should try to detect if we need to despawn the current visualization (i.e. cameras removed/added)
        // and only despawn the current camera position
        visualization.despawn(&*camera_system, &mut commands);
        visualization.spawn(
            &*camera_system,
            load_iterator.full_index_from_lod_index(pool.current_mesh_index),
            &mut commands,
            materials,
            meshes,
        );
    } else {
        visualization.despawn(&*camera_system, &mut commands);
    }

    if camera_system.follow_camera {
        if let Ok((_, mut transform)) = query.single_mut() {
            if let Some(timeline_transform) = camera_system.enabled_timeline().and_then(|ctl| {
                ctl.transform_at_frame(
                    load_iterator.full_index_from_lod_index(pool.current_mesh_index),
                )
            }) {
                *transform = CameraFrame::isometry_to_transform(timeline_transform);
                camera_system.current_transform = transform.clone();
            }
        }
    }
}

fn state_plumbing(mut camera_system: ResMut<CameraSystem>) {
    // println!("refresh current timeline");
    camera_system.refresh_current_timeline();
}
