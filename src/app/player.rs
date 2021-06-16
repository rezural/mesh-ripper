use std::{path::Path, time::Duration};

use super::{AppOptions, actions::{Actions, FrameDirection}, loading::FluidAssets, resources::wave_positions::WavePositions};
use super::GameState;
use bevy::{pbr::AmbientLight, prelude::*, render::camera::PerspectiveProjection};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};

pub struct PlayerPlugin;

pub struct Player;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_plugin(FlyCameraPlugin);

        app.add_system_set(
            SystemSet::on_enter(GameState::Playing)
                .with_system(spawn_camera.system())
                .with_system(spawn_world.system())
        )
        .add_system_set(SystemSet::on_update(GameState::Playing)
            .with_system(move_player.system())
            .with_system(check_for_reload.system())
        )

        
        .add_system_set(SystemSet::on_exit(GameState::Playing).with_system(remove_player.system()));
    }
}

fn spawn_camera(mut commands: Commands,
    mut ambient_light: ResMut<AmbientLight>,
) {
    ambient_light.color = Color::WHITE;
    ambient_light.brightness = 0.4;
    let fly_camera = FlyCamera {
        max_speed: 4.,
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

#[derive(Clone)]
pub struct FluidPool {
    pub num_fluids: usize,
    pub frame_direction: FrameDirection,
    current_fluid_index: usize,
    current_fluid_entity: Option<Entity>,
    current_mesh_handle: Option<Handle<Mesh>>,
    needs_update: bool,
    advance_every: Duration,
    currently_advanced: Duration,

    // current_fluid: &'a Handle<Mesh>,
}

impl FluidPool {
    pub fn new(num_fluids: usize, advance_every: Duration) -> Self {
        Self {
            num_fluids,
            advance_every,
            currently_advanced: Duration::default(),
            current_fluid_index: 0,
            current_fluid_entity: None,
            current_mesh_handle: None,
            needs_update: true,
            frame_direction: Default::default(),
        }
    }

    fn advance(&mut self) {
        assert!(self.num_fluids != 0);
        self.current_fluid_index = (self.current_fluid_index + 1) % self.num_fluids;
    }

    fn retreat(&mut self) {
        self.current_fluid_index = if self.current_fluid_index > 0 {
            self.current_fluid_index - 1
        } else {
            self.num_fluids - 1
        };

    }

    fn move_in_frame_direction(&mut self) {
        if let FrameDirection::Forward = self.frame_direction {
            self.advance();
        } else if let FrameDirection::Back = self.frame_direction {
            self.retreat();
        }

    }

    pub fn reset(&mut self) {
        self.current_fluid_index = 0;
    }

    pub fn needs_update(&self, delta: Duration) -> bool {
        if let FrameDirection::Paused = self.frame_direction {
            return false;
        }
        self.currently_advanced + delta > self.advance_every
    }

    pub fn update_fluid(&mut self, mut commands: Commands, fluids: FluidAssets, water_material: Handle<StandardMaterial>, delta: Duration) {
        if !self.needs_update(delta) {
            self.currently_advanced += delta;
            return;
        }

        self.currently_advanced = Duration::default();
        self.move_in_frame_direction();

        if self.current_fluid_entity.is_some() {
            let mut current_entity = commands.entity(self.current_fluid_entity.unwrap());
            current_entity.despawn_recursive();
        }

        let new_fluid = fluids
            .loaded
            .get(self.current_fluid_index)
            .unwrap()
            .clone();

        let entity = commands.spawn().insert_bundle(PbrBundle {
            mesh: new_fluid.1.clone(),
            material: water_material.clone(),
            transform: Transform {
                scale: Vec3::new(4., 4., 4.),
                ..Default::default()
            },
            ..Default::default()
        }).id();

        self.current_mesh_handle = Some(new_fluid.1);
        self.current_fluid_entity = Some(entity);

    }

    pub fn _update_position(&mut self,
        _fluid_assets: FluidAssets,
        _positions: WavePositions,
    ) -> Option<Vec3> {
        None
    }

}

fn spawn_world(
    mut commands: Commands,
    materials: ResMut<Assets<StandardMaterial>>,
    fluid_assets: Res<FluidAssets>,
    actions: Res<Actions>,
    time: Res<Time>,

) {

    let fluid_pool_length = fluid_assets.loaded.len();
    let mut pool = FluidPool::new(fluid_pool_length, actions.advance_every);
    commands.insert_resource(pool.clone());

    let water_material = materials.get_handle(fluid_assets.material.id);
    pool.update_fluid(commands, (*fluid_assets).clone(), water_material, time.delta());
    
}

fn move_player(
    commands: Commands,
    time: Res<Time>,
    mut actions: ResMut<Actions>,
    fluid_assets: Res<FluidAssets>,
    mut pool: ResMut<FluidPool>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    pool.advance_every = actions.advance_every;
    pool.frame_direction = actions.frame_direction.clone();

    if actions.reset {
        pool.reset();
        actions.reset = false;
    }
    pool.num_fluids = fluid_assets.loaded.len();

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
    fluid_assets: ResMut<FluidAssets>,
    config: Res<AppOptions>,
    asset_server: ResMut<AssetServer>,
) {
    if !actions.reload  {
        return;
    }

    add_extra_files_to_load(config, fluid_assets, asset_server);
    actions.reload = false;
}

fn add_extra_files_to_load(
    config: Res<AppOptions>,
    mut fluid_assets: ResMut<FluidAssets>,
    asset_server: ResMut<AssetServer>,

) {
    let glob = config.file_glob.as_str();

    let fluid_files: Vec<String> = glob::glob(glob)
        .expect("Loading fluid from assets failed in glob")
        .map(|entry| entry.unwrap().to_string_lossy().to_string())
        .collect();
    

    let not_already_loading = fluid_files
    .iter()
    .filter(|&file_name| 
        !fluid_assets.loading.iter().any(|(loaded_name, _)| 
            file_name == loaded_name
        ));

    let fluids_to_load = not_already_loading
        .filter(|&file_name| 
            !fluid_assets.loaded.iter().any(|(loaded_name, _)| 
                file_name == loaded_name
            ));

    let fluids_to_load: Vec<(String, HandleUntyped)> = fluids_to_load
        .map(|fluid_file| (fluid_file.clone(), asset_server.load_untyped(Path::new(&fluid_file).strip_prefix("assets/").unwrap())))
        .collect();

    
    fluid_assets.loading.extend(fluids_to_load);
}

fn remove_player(mut commands: Commands, player_query: Query<Entity, With<Player>>) {
    for player in player_query.iter() {
        commands.entity(player).despawn();
    }
}
