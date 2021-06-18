mod paths;

use std::{path::Path};

use super::{AppOptions, actions::Actions, loading::paths::PATHS};
use super::GameState;

use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy_kira_audio::AudioSource;

use rand::thread_rng;
use rand::seq::SliceRandom;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::on_enter(GameState::Loading).with_system(start_loading.system()),
        )
        .add_system_set(SystemSet::on_update(GameState::Loading).with_system(check_state.system()))
        .add_system_set(SystemSet::on_update(GameState::Playing).with_system(check_assets_ready.system()));

    }
}

pub struct LoadingState {
    textures: Vec<HandleUntyped>,
    fonts: Vec<HandleUntyped>,
    audio: Vec<HandleUntyped>,
    // fluids: Vec<(String, HandleUntyped)>,
}

#[derive(Clone)]
pub struct FluidAssets {
    pub loaded: Vec<(String, Handle<Mesh>)>,
    pub loading: Vec<(String, HandleUntyped)>,
    pub material: Handle<StandardMaterial>
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

fn start_loading(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    config: Res<AppOptions>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut fonts: Vec<HandleUntyped> = vec![];
    fonts.push(asset_server.load_untyped(PATHS.fira_sans));

    let mut audio: Vec<HandleUntyped> = vec![];
    audio.push(asset_server.load_untyped(PATHS.audio_flying));

    let mut textures: Vec<HandleUntyped> = vec![];
    textures.push(asset_server.load_untyped(PATHS.texture_bevy));

    let glob = config.file_glob.as_str();

    let mut fluid_files: Vec<String> = glob::glob(glob)
        .expect("Loading fluid from assets failed in glob")
        .map(|entry| entry.unwrap().to_string_lossy().to_string())
        .collect();

    alphanumeric_sort::sort_str_slice(fluid_files.as_mut());

    fluid_files.shuffle(&mut thread_rng());


    let fluids_to_load = fluid_files
        .iter()
        .map(|fluid_file| (fluid_file.clone(), asset_server.load_untyped(Path::new(fluid_file).strip_prefix("assets/").unwrap())))
        .collect();
    
    let water_colour = Color::rgba(95./255., 133./255., 194./255., 0.96);
    let material: Handle<StandardMaterial> = materials.add(water_colour.into());
    let water_material = materials.get_mut(material.clone());
    if let Some(water_material) = water_material {
        water_material.double_sided = true;
    }

    commands.insert_resource(FluidAssets {
        loaded: Vec::new(),
        material: material,
        loading: fluids_to_load,
    });

    commands.insert_resource(LoadingState {
        textures,
        fonts,
        audio,
    });


}


fn check_assets_ready(
    server: Res<AssetServer>,
    mut actions: ResMut<Actions>, 
    mut fluids: ResMut<FluidAssets>,
) {
    let loaded: Vec<(String, Handle<Mesh>)> = fluids.loading
        .iter()
        .filter(|(_, handle)| LoadState::Loaded == server.get_load_state(handle))
        .map(|(file, handle)| (file.clone(), server.get_handle(handle)))
        .collect();
    
    let loading: Vec<(String, HandleUntyped)> = fluids.loading
        .iter()
        .filter(|(_, handle)| ! (LoadState::Loaded == server.get_load_state(handle)))
        .cloned()
        .collect();


    actions.fluids_loaded = fluids.loaded.len();
    actions.fluids_loaded_percent = (fluids.loaded.len().max(1) as f32 / (fluids.loaded.len() + loading.len()) as f32)  * 100.;

    fluids.loaded.extend(loaded);
    fluids.loaded.sort_by(|(a,_), (b,_)| 
        alphanumeric_sort::compare_str(a.as_str(), b.as_str()));
    
    fluids.loading = loading;

}

fn check_state(
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

    state.set(GameState::Menu).unwrap();

}
