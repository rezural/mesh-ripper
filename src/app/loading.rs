mod paths;

use std::path::Path;

use super::resources::background_meshes::BackgroundMeshes;
use super::GameState;
use super::{
    loading::paths::PATHS, resources::lod_midpoint_iterator::MidpointIterator, AppOptions,
};
use crate::app::inspector::vec_as_dropdown::VecAsDropdown;
use crate::app::resources::actions::Actions;
use crate::app::resources::asset_load_checker::{AssetLoadChecker, LoadingSource};
use crate::app::resources::glob_or_dir_loader::GlobOrDirLoader;
use crate::app::resources::load_manager::LoadManager;
use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy_kira_audio::AudioSource;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(
        &self,
        app: &mut AppBuilder,
    ) {
        app.add_system_set(
            SystemSet::on_enter(GameState::RegisterInitialResources).with_system(
                register_initial_resources
                    .system()
                    .label("register_initial_resources"),
            ),
        );

        // needs to run after create_load_manager (next frame, unless we use stages)
        app.add_system_set(
            SystemSet::on_enter(GameState::Loading)
                .with_system(load_mesh_assets.system().label("load_mesh_assets"))
                .with_system(load_assets.system().after("load_mesh_assets")),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Loading).with_system(check_assets.system()),
        );
        // .add_system_set(
        //     SystemSet::on_update(GameState::Playing).with_system(check_mesh_assets.system()),
        // );
    }
}

pub struct LoadingState {
    textures: Vec<HandleUntyped>,
    fonts: Vec<HandleUntyped>,
    audio: Vec<HandleUntyped>,
    // fluids: Vec<(String, HandleUntyped)>,
}

#[derive(Clone)]
pub struct MeshAssets {
    pub loaded: Vec<(String, Handle<Mesh>)>,
    pub loading: Vec<(String, HandleUntyped)>,
    pub material: Handle<StandardMaterial>,
}
pub struct FontAssets {
    pub fira_sans: Handle<Font>,
}

pub struct AudioAssets {
    pub flying: Handle<AudioSource>,
}

pub struct TextureAssets {
    pub texture_bevy: Handle<Texture>,
}

fn register_initial_resources(
    mut commands: Commands,
    config: Res<AppOptions>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<State<GameState>>,
    asset_server: Res<AssetServer>,
) {
    // load files
    let load_files: Vec<String> = Vec::new();
    let load_iterator = MidpointIterator::new(load_files, config.load_max);
    let load_manager = LoadManager::new(load_iterator);
    let mut glob_or_dir_loader = GlobOrDirLoader::new(
        load_manager,
        config.file_glob.clone(),
        config.dataset_dir.clone(),
    );

    glob_or_dir_loader.update(config.file_glob.clone(), None, &asset_server);

    commands.insert_resource(glob_or_dir_loader.clone());

    // Background meshes

    let background_meshes = BackgroundMeshes::default();
    commands.insert_resource(background_meshes);
    let asset_load_checker: AssetLoadChecker<Mesh> = AssetLoadChecker::new();

    commands.insert_resource(asset_load_checker);

    // set the water color
    let water_colour = Actions::default().fluid_color;
    let material: Handle<StandardMaterial> = materials.add(water_colour.into());

    let water_material = materials.get_mut(material.clone());
    if let Some(water_material) = water_material {
        water_material.double_sided = true;
        water_material.roughness = 0.1;
    }

    let load_manager = glob_or_dir_loader.load_manager();
    let mut actions = Actions::default();
    actions.load_number_of_frames = VecAsDropdown::new(load_manager.load_iterator.get_lods());

    let mut dataset_dirs: Vec<String> = vec![String::from("Choose Datadir")];
    if let Some(entries) = glob_or_dir_loader.dirs_from_load_dir() {
        dataset_dirs.extend(entries);
    }

    actions.datasets = VecAsDropdown::new(dataset_dirs);

    commands.insert_resource(actions);

    commands.insert_resource(MeshAssets {
        loaded: load_manager.loaded.clone(),
        loading: load_manager.loading.clone(),
        material: material,
    });

    state.set(GameState::Loading).unwrap();
}

fn load_mesh_assets(
    mut glob_or_dir_loader: ResMut<GlobOrDirLoader>,
    asset_server: Res<AssetServer>,
    config: Res<AppOptions>,
    mut background_meshes: ResMut<BackgroundMeshes>,
) {
    glob_or_dir_loader
        .load_manager_mut()
        .load_assets(&asset_server);

    if let Some(load_mesh) = config.load_mesh.clone() {
        let handle =
            asset_server.load_untyped(Path::new(&load_mesh).strip_prefix("assets/").unwrap());
        (*background_meshes).loading_mut().push(handle);
    }
}

fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let mut fonts: Vec<HandleUntyped> = vec![];
    fonts.push(asset_server.load_untyped(PATHS.fira_sans));

    let mut audio: Vec<HandleUntyped> = vec![];
    audio.push(asset_server.load_untyped(PATHS.audio_flying));

    let mut textures: Vec<HandleUntyped> = vec![];
    textures.push(asset_server.load_untyped(PATHS.texture_bevy));

    commands.insert_resource(LoadingState {
        textures,
        fonts,
        audio,
    });
}

fn check_assets(
    mut commands: Commands,
    mut state: ResMut<State<GameState>>,
    asset_server: Res<AssetServer>,
    loading_state: Res<LoadingState>,
) {
    if LoadState::Loaded
        != asset_server.get_group_load_state(loading_state.fonts.iter().map(|handle| handle.id))
    {
        return;
    }
    if LoadState::Loaded
        != asset_server.get_group_load_state(loading_state.textures.iter().map(|handle| handle.id))
    {
        return;
    }
    if LoadState::Loaded
        != asset_server.get_group_load_state(loading_state.audio.iter().map(|handle| handle.id))
    {
        return;
    }

    commands.insert_resource(FontAssets {
        fira_sans: asset_server.get_handle(PATHS.fira_sans),
    });

    commands.insert_resource(AudioAssets {
        flying: asset_server.get_handle(PATHS.audio_flying),
    });

    commands.insert_resource(TextureAssets {
        texture_bevy: asset_server.get_handle(PATHS.texture_bevy),
    });

    state.set(GameState::Playing).unwrap();
}
