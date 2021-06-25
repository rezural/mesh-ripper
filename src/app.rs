pub mod actions;
pub mod audio;
pub mod loading;
pub mod menu;
pub mod player;
pub mod resources;
pub mod inspector;

use actions::ActionsPlugin;
use audio::InternalAudioPlugin;
use loading::LoadingPlugin;
use menu::MenuPlugin;
use player::PlayerPlugin;
use actions::Actions;

use bevy::app::AppBuilder;
// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_inspector_egui::InspectorPlugin;
use bevy_obj::*;

use structopt::StructOpt;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    Loading,
    Playing,
    Menu,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "Options")]
pub struct AppOptions { 
    file_glob: String,
    #[structopt(short, long, default_value = "100")]
    load_max: usize,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_state(GameState::Loading)
            .add_plugin(LoadingPlugin)
            .add_plugin(ActionsPlugin)
            .add_plugin(MenuPlugin)
            .add_plugin(InternalAudioPlugin)
            .add_plugin(PlayerPlugin)
            .add_plugin(ObjPlugin)
            .add_plugin(InspectorPlugin::<Actions>::new())
            // .add_plugin(FrameTimeDiagnosticsPlugin::default())
            // .add_plugin(LogDiagnosticsPlugin::default())
            ;
    }
}
