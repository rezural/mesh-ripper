use std::time::Duration;

use crate::app::loading::MeshAssets;
use crate::app::resources::actions::FrameDirection;
use crate::support::loader_fu::features::Features;
use crate::support::loader_fu::render::{FeatureAwareRenderer, PointRenderOptions, RenderCache};
use bevy::prelude::*;

#[derive(Clone)]
pub struct MeshPool {
    pub num_fluids: usize,
    pub frame_direction: FrameDirection,
    pub advance_every: Duration,
    pub paused: bool,
    pub current_mesh_index: usize,
    pub have_displayed: bool,
    pub sample_size: usize,
    current_fluid_entities: Option<Vec<Entity>>,
    current_mesh_handle: Option<Handle<Mesh>>,
    needs_update: bool,
    currently_advanced: Duration,
    previous_mesh_size: usize,
    sampled_indices: Vec<usize>,
    // current_fluid: &'a Handle<Mesh>,
}

impl MeshPool {
    pub fn new(
        num_fluids: usize,
        advance_every: Duration,
        sample_size: usize,
    ) -> Self {
        Self {
            num_fluids,
            advance_every,
            currently_advanced: Duration::default(),
            paused: true,
            have_displayed: false,
            current_mesh_index: 0,
            current_fluid_entities: None,
            current_mesh_handle: None,
            needs_update: true,
            frame_direction: Default::default(),
            sampled_indices: Vec::new(),
            previous_mesh_size: 0,
            sample_size,
        }
    }

    pub fn advance(&mut self) {
        // assert!(self.num_fluids != 0);
        if self.num_fluids > 0 {
            self.current_mesh_index = (self.current_mesh_index + 1) % self.num_fluids;
        }
    }

    pub fn retreat(&mut self) {
        self.current_mesh_index = if self.current_mesh_index > 0 {
            self.current_mesh_index - 1
        } else {
            self.num_fluids - 1
        };
    }

    fn update_previous_mesh_size(
        &mut self,
        mesh: &Mesh,
    ) {
        let features = Features::new(mesh);

        if let Some(vertices) = features.vertices() {
            self.previous_mesh_size = vertices.len();
        } else {
            self.previous_mesh_size = 0;
        }
    }

    fn move_in_frame_direction(&mut self) {
        if self.paused {
            return;
        }
        if let FrameDirection::Forward = self.frame_direction {
            self.advance();
        } else if let FrameDirection::Back = self.frame_direction {
            self.retreat();
        }
    }

    pub fn reset(&mut self) {
        self.current_mesh_index = 0;
    }

    pub fn needs_update(
        &self,
        delta: Duration,
    ) -> bool {
        if !self.have_displayed {
            return true;
        };
        if self.paused {
            return false;
        }
        self.currently_advanced + delta > self.advance_every
    }

    pub fn current_mesh<'a>(
        &self,
        fluids: &'a MeshAssets,
    ) -> Option<&'a (String, Handle<Mesh>)> {
        fluids.loaded.get(self.current_mesh_index)
    }

    pub fn despawn_mesh(
        &mut self,
        commands: &mut Commands,
        meshes: &Assets<Mesh>,
    ) {
        if let Some(current_mesh_handle) = self.current_mesh_handle.clone() {
            if let Some(current_entities) = self.current_fluid_entities.as_deref() {
                let renderer = FeatureAwareRenderer::new(current_mesh_handle);
                renderer.despawn(commands, meshes, current_entities.to_vec());
            }
        }
    }

    pub fn spawn_mesh(
        &mut self,
        fluids: &MeshAssets,
        water_material: Handle<StandardMaterial>,
        commands: &mut Commands,
        meshes: &Assets<Mesh>,
        render_cache: &RenderCache,
        render_style: PointRenderOptions,
    ) {
        if let Some(new_fluid) = self.current_mesh(fluids) {
            let mut mesh_size_changed = false;
            if let Some(mesh) = meshes.get(new_fluid.1.clone()) {
                let features = Features::new(mesh);
                if let Some(vertices) = features.vertices() {
                    if vertices.len() != self.previous_mesh_size {
                        mesh_size_changed = true;
                    }
                } else {
                    mesh_size_changed = true;
                }
            }
            if self.sampled_indices.len() != self.sample_size || mesh_size_changed {
                if let Some(actual_mesh) = meshes.get(new_fluid.1.clone()) {
                    self.sampled_indices =
                        FeatureAwareRenderer::sample_indices(actual_mesh, self.sample_size);
                }
            }

            let renderer = FeatureAwareRenderer::new(new_fluid.1.clone());
            self.current_fluid_entities = Some(renderer.spawn(
                commands,
                meshes,
                water_material,
                render_style,
                render_cache,
                &self.sampled_indices,
            ));
            if self.current_fluid_entities.is_some() {
                self.current_mesh_handle = Some(new_fluid.1.clone());
            }
            if let Some(new_fluid) = self.current_mesh(fluids) {
                if let Some(mesh) = meshes.get(new_fluid.1.clone()) {
                    self.update_previous_mesh_size(mesh);
                }
            }
        }
    }

    pub fn clear(
        &mut self,
        commands: &mut Commands,
        meshes: &Assets<Mesh>,
    ) {
        self.despawn_mesh(commands, meshes);
        self.current_mesh_handle = None;
        self.current_mesh_index = 0;
        self.sampled_indices.clear();
    }

    pub fn redraw(
        &mut self,
        commands: &mut Commands,
        fluids: &MeshAssets,
        water_material: Handle<StandardMaterial>,
        render_cache: &RenderCache,
        meshes: &Assets<Mesh>,
        render_style: PointRenderOptions,
    ) {
        self.despawn_mesh(commands, meshes);
        self.spawn_mesh(
            fluids,
            water_material,
            commands,
            meshes,
            render_cache,
            render_style,
        );
    }

    pub fn update_fluid(
        &mut self,
        commands: &mut Commands,
        fluids: &MeshAssets,
        water_material: Handle<StandardMaterial>,
        delta: Duration,
        render_cache: &RenderCache,
        meshes: &Assets<Mesh>,
        render_style: PointRenderOptions,
        sample_size: usize,
    ) {
        if !self.needs_update(delta) {
            self.currently_advanced += delta;
            return;
        }

        self.currently_advanced = Duration::default();

        if fluids.loaded.len() > 0 {
            self.despawn_mesh(commands, meshes);
            self.move_in_frame_direction();

            self.spawn_mesh(
                fluids,
                water_material,
                commands,
                meshes,
                render_cache,
                render_style,
            );
            self.have_displayed = true;
        }
    }
}
