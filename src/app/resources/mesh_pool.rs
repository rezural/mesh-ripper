use std::time::Duration;

use crate::app::{actions::FrameDirection, loading::MeshAssets};
use bevy::prelude::*;

#[derive(Clone)]
pub struct MeshPool {
    pub num_fluids: usize,
    pub frame_direction: FrameDirection,
    pub advance_every: Duration,
    pub paused: bool,
    pub current_mesh_index: usize,
    pub have_displayed: bool,
    current_fluid_entity: Option<Entity>,
    current_mesh_handle: Option<Handle<Mesh>>,
    needs_update: bool,
    currently_advanced: Duration,
    // current_fluid: &'a Handle<Mesh>,
}

impl MeshPool {
    pub fn new(
        num_fluids: usize,
        advance_every: Duration,
    ) -> Self {
        Self {
            num_fluids,
            advance_every,
            currently_advanced: Duration::default(),
            paused: true,
            have_displayed: false,
            current_mesh_index: 0,
            current_fluid_entity: None,
            current_mesh_handle: None,
            needs_update: true,
            frame_direction: Default::default(),
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
    ) {
        if let Some(entity) = self.current_fluid_entity {
            let mut current_entity = commands.entity(entity);
            current_entity.despawn_recursive();
            self.current_fluid_entity = None;
        }
    }

    pub fn spawn_mesh(
        &mut self,
        fluids: &MeshAssets,
        water_material: Handle<StandardMaterial>,
        commands: &mut Commands,
    ) {
        if let Some(new_fluid) = self.current_mesh(fluids) {
            let entity = commands
                .spawn()
                .insert_bundle(PbrBundle {
                    mesh: new_fluid.1.clone(),
                    material: water_material.clone(),
                    // transform: Transform {
                    //     scale: Vec3::new(1., 4., 4.),
                    //     ..Default::default()
                    // },
                    ..Default::default()
                })
                .id();

            self.current_mesh_handle = Some(new_fluid.1.clone());
            self.current_fluid_entity = Some(entity);
        }
    }

    pub fn redraw(
        &mut self,
        commands: &mut Commands,
        fluids: &MeshAssets,
        water_material: Handle<StandardMaterial>,
    ) {
        self.despawn_mesh(commands);
        self.spawn_mesh(fluids, water_material, commands);
    }

    pub fn update_fluid(
        &mut self,
        commands: &mut Commands,
        fluids: &MeshAssets,
        water_material: Handle<StandardMaterial>,
        delta: Duration,
    ) {
        if !self.needs_update(delta) {
            self.currently_advanced += delta;
            return;
        }

        self.currently_advanced = Duration::default();

        if fluids.loaded.len() > 0 {
            self.despawn_mesh(commands);
            self.move_in_frame_direction();

            self.spawn_mesh(fluids, water_material, commands);
            self.have_displayed = true;
        }
    }
}
