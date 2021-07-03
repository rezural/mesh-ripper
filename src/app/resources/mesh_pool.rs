use std::time::Duration;

use crate::app::{actions::FrameDirection, loading::MeshAssets};
use bevy::prelude::*;

use super::wave_positions::WavePositions;

#[derive(Clone)]
pub struct MeshPool {
    pub num_fluids: usize,
    pub frame_direction: FrameDirection,
    pub advance_every: Duration,
    pub paused: bool,
    have_displayed: bool,
    current_fluid_index: usize,
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
            current_fluid_index: 0,
            current_fluid_entity: None,
            current_mesh_handle: None,
            needs_update: true,
            frame_direction: Default::default(),
        }
    }

    fn advance(&mut self) {
        assert!(self.num_fluids != 0);
        self.current_fluid_index = (self.current_fluid_index + 1) % self.num_fluids;
    }

    fn retreat(&mut self) {
        self.current_fluid_index = if self.current_fluid_index > 0 {
            self.current_fluid_index - 1
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
        self.current_fluid_index = 0;
    }

    pub fn needs_update(
        &self,
        delta: Duration,
    ) -> bool {
        if !self.have_displayed {
            return true;
        }
        if self.paused {
            return false;
        }
        self.currently_advanced + delta > self.advance_every
    }

    pub fn update_fluid(
        &mut self,
        mut commands: Commands,
        fluids: MeshAssets,
        water_material: Handle<StandardMaterial>,
        delta: Duration,
    ) {
        if !self.needs_update(delta) {
            self.currently_advanced += delta;
            return;
        }

        self.currently_advanced = Duration::default();
        self.move_in_frame_direction();

        if self.current_fluid_entity.is_some() {
            let mut current_entity = commands.entity(self.current_fluid_entity.unwrap());
            current_entity.despawn_recursive();
        }

        if fluids.loaded.len() > 0 {
            self.have_displayed = true;

            let new_fluid = fluids.loaded.get(self.current_fluid_index).unwrap().clone();

            let entity = commands
                .spawn()
                .insert_bundle(PbrBundle {
                    mesh: new_fluid.1.clone(),
                    material: water_material.clone(),
                    transform: Transform {
                        scale: Vec3::new(4., 4., 4.),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .id();

            self.current_mesh_handle = Some(new_fluid.1);
            self.current_fluid_entity = Some(entity);
        }
    }

    pub fn _update_position(
        &mut self,
        _fluid_assets: MeshAssets,
        _positions: WavePositions,
    ) -> Option<Vec3> {
        None
    }
}
