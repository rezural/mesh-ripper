use super::resources::actions::{Actions, State as AppState};
use super::resources::camera::*;
use super::GameState;
use bevy::prelude::*;
pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(
        &self,
        app: &mut AppBuilder,
    ) {
        app.add_system_set(
            SystemSet::on_update(GameState::Playing).with_system(state_plumbing.system()),
        );
        app.init_resource::<Actions>().init_resource::<AppState>();
    }
}

fn state_plumbing(mut camera_system: ResMut<CameraSystem>) {
    // println!("refresh current timeline");
    camera_system.refresh_current_timeline();
}
