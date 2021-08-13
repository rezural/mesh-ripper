use crate::support::loader_fu::render::{FeatureAwareRenderer, RenderCache};

use super::resources::actions::{Actions, State as AppState};
use super::resources::camera::*;
use super::resources::mesh_pool::MeshPool;
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

fn state_plumbing(
    mut camera_system: ResMut<CameraSystem>,
    actions: Res<Actions>,
    mut render_cache: ResMut<RenderCache>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_pool: ResMut<MeshPool>,
) {
    camera_system.refresh_current_timeline();

    if actions.particle_radius != render_cache.radius {
        FeatureAwareRenderer::cache_meshes(
            &mut *meshes,
            actions.particle_radius,
            &mut *render_cache,
        );
    }

    if actions.max_particles_render != mesh_pool.sample_size {
        mesh_pool.sample_size = actions.max_particles_render;
    }
}
