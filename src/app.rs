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
use bevy_gizmos::*;
use bevy_inspector_egui::InspectorPlugin;
use bevy_obj::*;
use bevy_ply::*;

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

#[derive(Default)]
struct State {
    pub gizmo_entity: Option<Entity>,
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
        app.add_plugin(ObjPlugin)
            .add_plugin(bevy_stl::StlPlugin)
            .add_plugin(PlyPlugin);

        app.add_system(persistent_gizmos.system());
        app.add_startup_system(initialize_state.system());
    }
}

fn initialize_state(mut commands: Commands) {
    commands.insert_resource(State::default())
}

fn persistent_gizmos(
    mut commands: Commands,
    actions: ResMut<Actions>,
    mut state: ResMut<State>,
) {
    if actions.show_axis {
        let entity = commands
            .spawn()
            .insert_bundle(GizmoBundle {
                transform: Transform::from_xyz(-4.0, 1.5, 0.0),
                gizmo: Gizmo {
                    shape: GizmoShape::Empty { radius: 400.0 },
                    wireframe: Color::rgba(1.0, 1.0, 0.0, 1.0),
                    color: Color::rgba(0.6, 0.8, 0.2, 0.0),
                },
                ..Default::default()
            })
            .id();
        state.gizmo_entity = Some(entity);
    } else {
        if let Some(entity) = state.gizmo_entity {
            commands.entity(entity).despawn_recursive();
            state.gizmo_entity = None;
        }
    }
}
