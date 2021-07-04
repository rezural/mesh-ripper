pub mod actions;
pub mod inspector;
pub mod loading;
pub mod menu;
pub mod player;
pub mod resources;

use actions::Actions;
use actions::ActionsPlugin;
use loading::LoadingPlugin;
use menu::MenuPlugin;
use player::PlayerPlugin;

use bevy::app::AppBuilder;
// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_inspector_egui::InspectorPlugin;
use bevy_obj::*;

use structopt::StructOpt;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    RegisterInitialResources,
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
    fn build(
        &self,
        app: &mut AppBuilder,
    ) {
        app.add_state(GameState::RegisterInitialResources)
            .add_plugin(LoadingPlugin)
            .add_plugin(ActionsPlugin)
            .add_plugin(MenuPlugin)
            .add_plugin(PlayerPlugin)
            .add_plugin(InspectorPlugin::<Actions>::new())
            // .add_plugin(FrameTimeDiagnosticsPlugin::default())
            // .add_plugin(LogDiagnosticsPlugin::default())
            ;
        app
            .add_plugin(ObjPlugin)
            .add_plugin(bevy_stl::StlPlugin);
    }
}
