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
use bevy_gizmos::{Axis, *};
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
    #[structopt(short, long)]
    file_glob: Option<String>,
    #[structopt(short, long, default_value = "assets/data")]
    dataset_dir: String,
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
            .add_plugin(GizmosPlugin)
            // .add_plugin(FrameTimeDiagnosticsPlugin::default())
            // .add_plugin(LogDiagnosticsPlugin::default())
            ;
        app.add_plugin(ObjPlugin).add_plugin(bevy_stl::StlPlugin);
        app.add_system(persistent_gizmos.system());
    }
}

fn persistent_gizmos(
    mut commands: Commands,
    actions: ResMut<Actions>,
) {
    if actions.show_axis {
        commands.spawn().insert_bundle(GizmoBundle {
            transform: Transform::from_xyz(-4.0, 1.5, 0.0),
            gizmo: Gizmo {
                shape: GizmoShape::Empty { radius: 400.0 },
                wireframe: Color::rgba(1.0, 1.0, 0.0, 1.0),
                color: Color::rgba(0.6, 0.8, 0.2, 0.2),
            },
            ..Default::default()
        });
    }
}
