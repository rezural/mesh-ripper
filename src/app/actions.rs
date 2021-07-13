use super::inspector::vec_as_dropdown::VecAsDropdown;
use super::resources::camera::*;
use super::resources::glob_or_dir_loader::GlobOrDirLoader;
use super::resources::mesh_pool::MeshPool;
use super::GameState;
use bevy::prelude::*;
use bevy_fly_camera::FlyCamera;
use bevy_inspector_egui::Inspectable;

pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(
        &self,
        app: &mut AppBuilder,
    ) {
        app.add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(set_movement_actions.system())
                .with_system(camera_timeline_system.system()),
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
    pub camera_mode: bool,
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
            camera_mode: false,
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

// impl Default for CameraSystem {}

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
            actions.advance_every -= 0.01;
        } else {
            actions.advance_every -= 0.001;
        }
    }

    if keyboard_input.just_pressed(KeyCode::G) {
        if actions.advance_every > 0.019 {
            actions.advance_every += 0.01;
        } else {
            actions.advance_every += 0.001;
        }
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
    actions: Res<Actions>,
) {
    if camera_system.record_mode {
        // we need the highest LOD
        let load_manager = (&mut *loader).load_manager_mut();
        load_manager.highest_lod();
        load_manager.reload(&*asset_server);
        if let Some(timeline) = camera_system.enabled_timeline_mut() {
            if keyboard_input.just_pressed(KeyCode::C) {
                if let Ok((_, transform)) = query.single_mut() {
                    let camera_pose = transform;
                    timeline.add_frame(pool.current_mesh_index, *camera_pose);
                }
            }
        }
    }

    if camera_system.show_camera_visualization {
        visualization.despawn(&*camera_system, &mut commands);
        visualization.spawn(
            &*camera_system,
            pool.current_mesh_index,
            &mut commands,
            materials,
            meshes,
        );
    } else {
        visualization.despawn(&*camera_system, &mut commands);
    }

    if camera_system.follow_camera {
        if let Ok((mut camera, mut transform)) = query.single_mut() {
            if actions.paused {
                if !camera.enabled {
                    (*camera).enabled = true;
                }
            } else {
                (*camera).enabled = false;
                if let Some(timeline_transform) = camera_system
                    .enabled_timeline()
                    .and_then(|ctl| ctl.transform_at_frame(pool.current_mesh_index))
                {
                    *transform = timeline_transform;
                }
            }
        }
    }
}
