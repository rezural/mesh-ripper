use super::loading::MeshAssets;
use super::resources::actions::Actions;
use super::resources::camera::*;
use super::resources::glob_or_dir_loader::GlobOrDirLoader;
use super::resources::lod_midpoint_iterator::MidpointIterator;
use super::resources::mesh_lookat_estimator::MeshLookAtEstimator;
use super::resources::mesh_pool::MeshPool;
use super::GameState;
use bevy::prelude::*;
use smooth_bevy_cameras::controllers::fps::FpsCameraController;
use smooth_bevy_cameras::LookTransform;

pub struct RenderPlugin;

pub struct Render;

impl Plugin for RenderPlugin {
    fn build(
        &self,
        app: &mut AppBuilder,
    ) {
        app.add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(update_mesh.system().label("update_mesh"))
                .with_system(camera_timeline_system.system().before("update_mesh")),
        );
    }
}

fn update_mesh(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut actions: ResMut<Actions>,
    mut pool: ResMut<MeshPool>,
    camera_system: ResMut<CameraSystem>,
    fluid_assets: ResMut<MeshAssets>,
    loader: ResMut<GlobOrDirLoader>,
    meshes: Res<Assets<Mesh>>,
    mut transform_query: Query<(&mut FpsCameraController, &mut LookTransform, &mut Transform)>,
    time: Res<Time>,
) {
    if let Some(current_mesh) = pool.current_mesh(&fluid_assets) {
        if !pool.have_displayed {
            if let Some(mesh) = meshes.get(current_mesh.1.clone()) {
                if let Ok((_, mut transform, _)) = transform_query.single_mut() {
                    if let Some((eye, target)) = MeshLookAtEstimator::eye_target(mesh) {
                        transform.eye = eye;
                        transform.target = target;
                    }
                }
            }
        }
        actions.current_file = current_mesh.0.clone();
        actions.current_frame = pool.current_mesh_index;
    }

    // println!("update_mesh");
    let material = materials.get_handle(fluid_assets.material.id);
    let material = materials.get_mut(material.clone());

    if let Some(material) = material {
        material.base_color = actions.fluid_color;
        material.base_color.set_a(actions.opacity);
        material.double_sided = true;
        material.roughness = actions.material_roughness;
        let material = materials.get_handle(fluid_assets.material.id);
        // if let Ok((controller, mut transform, mut bevy_transform)) = look_query.single_mut() {
        //     println!("updaating camera");
        //     let load_iterator = &loader.load_manager().load_iterator;
        //     update_camera_system_transform(
        //         &*camera_system,
        //         controller,
        //         &mut *transform,
        //         &mut *bevy_transform,
        //         load_iterator,
        //         &*pool,
        //     );
        // }
        pool.update_fluid(&mut commands, &fluid_assets, material, time.delta());

        if let Some(current_mesh) = pool.current_mesh(&fluid_assets) {
            actions.current_file = current_mesh.0.clone();
        }
    }
}

fn update_camera_system_transform(
    camera_system: &CameraSystem,
    camera_controller: &mut FpsCameraController,
    transform: &mut LookTransform,
    bevy_transform: &mut Transform,
    load_iterator: &MidpointIterator<String>,
    pool: &MeshPool,
) {
    if camera_system.follow_camera {
        if let Some(timeline_transform) = camera_system.enabled_timeline().and_then(|ctl| {
            ctl.transform_at_frame(load_iterator.full_index_from_lod_index(pool.current_mesh_index))
        }) {
            let camera_transform = CameraFrame::isometry_to_transform(timeline_transform);
            // (*bevy_transform) = camera_transform;

            let eye = camera_transform.translation;
            let change = camera_transform.rotation.mul_vec3(-Vec3::Z);
            let target = eye + change;

            transform.eye = eye;
            transform.target = target;
        }
    }
}

fn camera_timeline_system(
    mut commands: Commands,
    materials: ResMut<Assets<StandardMaterial>>,
    meshes: ResMut<Assets<Mesh>>,
    mut camera_system: ResMut<CameraSystem>,
    mut visualization: ResMut<CameraSystemVisualization>,
    mut transform_query: Query<(&mut FpsCameraController, &mut LookTransform, &mut Transform)>,
    keyboard_input: Res<Input<KeyCode>>,
    mut loader: ResMut<GlobOrDirLoader>,
    pool: ResMut<MeshPool>,
    asset_server: Res<AssetServer>,
    mut actions: ResMut<Actions>,
    fluid_assets: ResMut<MeshAssets>,
    // time: Res<Time>,
) {
    // println!("camera-timeline-system");
    if camera_system.record_mode {
        // we need the highest LOD
        let load_manager = (&mut *loader).load_manager_mut();
        load_manager.reload(&*asset_server);
        let load_iterator = &loader.load_manager().load_iterator;

        if let Some(timeline) = camera_system.enabled_timeline_mut() {
            if keyboard_input.just_pressed(KeyCode::C) && !keyboard_input.pressed(KeyCode::LControl)
            {
                if let Ok((_, _, transform)) = transform_query.single_mut() {
                    timeline.add_frame(
                        load_iterator.full_index_from_lod_index(pool.current_mesh_index),
                        *transform,
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
                if let Ok((_, mut transform, _)) = transform_query.single_mut() {
                    if let Some((eye, target)) = MeshLookAtEstimator::eye_target(mesh) {
                        transform.eye = eye;
                        transform.target = target;
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

    if let Ok((mut controller, mut transform, mut bevy_transform)) = transform_query.single_mut() {
        // println!("updating camera");
        let load_iterator = &loader.load_manager().load_iterator;
        update_camera_system_transform(
            &*camera_system,
            &mut *controller,
            &mut *transform,
            &mut *bevy_transform,
            load_iterator,
            &*pool,
        );
    }
}
