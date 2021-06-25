mod paths;

use super::GameState;
use super::{
    actions::Actions, loading::paths::PATHS, resources::lod_midpoint_iterator::MidpointIterator,
    AppOptions,
};
use crate::app::actions::State as AppState;
use crate::app::inspector::vec_as_dropdown::VecAsDropdown;
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
            SystemSet::on_enter(GameState::Loading)
                .with_system(create_load_manager.system())
                .with_system(load_mesh_assets.system())
                .with_system(load_assets.system()),
        )
        .add_system_set(SystemSet::on_update(GameState::Loading).with_system(check_assets.system()))
        .add_system_set(
            SystemSet::on_update(GameState::Playing).with_system(check_mesh_assets.system()),
        );
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

fn create_load_manager(
    mut commands: Commands,
    config: Res<AppOptions>,
) {
    // load files
    let glob = config.file_glob.as_str();

    let mut fluid_files: Vec<String> = glob::glob(glob)
        .expect("Loading fluid from assets failed in glob")
        .map(|entry| entry.unwrap().to_string_lossy().to_string())
        .collect();

    //FIXME: sorting should be done in the load_manager
    alphanumeric_sort::sort_str_slice(fluid_files.as_mut());

    let fluid_files: MidpointIterator<String> = MidpointIterator::new(fluid_files, config.load_max);

    let load_manager = LoadManager::new(fluid_files.clone());
    commands.insert_resource(load_manager.clone());
}

fn load_mesh_assets(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut load_manager: ResMut<LoadManager>,
    asset_server: Res<AssetServer>,
) {
    // set the water color
    let water_colour = Actions::default().fluid_color;
    let material: Handle<StandardMaterial> = materials.add(water_colour.into());

    let water_material = materials.get_mut(material.clone());
    if let Some(water_material) = water_material {
        water_material.double_sided = true;
    }

    load_manager.load_assets(&asset_server);

    // get file_glob lods
    let mut actions = Actions::default();
    actions.lods = VecAsDropdown::new(load_manager.load_iterator.get_lods());
    commands.insert_resource(actions);

    commands.insert_resource(MeshAssets {
        loaded: load_manager.loaded.clone(),
        loading: load_manager.loading.clone(),
        material: material,
    });

    // FIXME: this should probably take a ref
    let state = AppState::new(load_manager.load_iterator.clone());
    commands.insert_resource(state);
}

fn check_mesh_assets(
    mut actions: ResMut<Actions>,
    mut fluids: ResMut<MeshAssets>,
    mut load_manager: ResMut<LoadManager>,
    asset_server: Res<AssetServer>,
) {
    load_manager.update_load_state(&asset_server);

    actions.fluids_loaded = fluids.loaded.len();
    actions.fluids_loaded_percent = (fluids.loaded.len().max(1) as f32
        / (fluids.loaded.len() + load_manager.loading.len()) as f32)
        * 100.;

    fluids.loaded.extend(load_manager.loaded.clone());
    fluids
        .loaded
        .sort_by(|(a, _), (b, _)| alphanumeric_sort::compare_str(a.as_str(), b.as_str()));

    fluids.loading = load_manager.loading.clone();
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
