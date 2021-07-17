use bevy::prelude::*;

use super::{camera_system::CameraSystem, CameraFrame};

#[derive(Default)]
pub struct CameraSystemVisualization {
    entities: Vec<Entity>,
    current_position_entities: Vec<Entity>,
}

impl CameraSystemVisualization {
    pub fn spawn(
        &mut self,
        camera_system: &CameraSystem,
        frame: usize,
        commands: &mut Commands,
        mut materials: ResMut<Assets<StandardMaterial>>,
        mut meshes: ResMut<Assets<Mesh>>,
    ) {
        if self.entities.len() > 0 {
            return;
        }
        if let Some(tl) = camera_system.enabled_timeline() {
            let poses = tl.timeline.iter().map(|t| t.pose);
            let entities: Vec<Entity> = poses
                .map(|p| {
                    commands
                        .spawn()
                        .insert_bundle(PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.4 })),
                            material: materials.add(Color::rgba(0.5, 0.5, 1.0, 0.5).into()),
                            transform: CameraFrame::isometry_to_transform(p),
                            ..Default::default()
                        })
                        .id()
                })
                .collect();
            self.entities.extend(entities);
        }

        if let Some(lerp) = camera_system
            .enabled_timeline()
            .and_then(|cs| cs.transform_at_frame(frame))
        {
            let id = commands
                .spawn()
                .insert_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 0.4 })),
                    material: materials.add(Color::rgba(0.5, 0.1, 1.0, 0.7).into()),
                    transform: CameraFrame::isometry_to_transform(lerp),
                    ..Default::default()
                })
                .id();
            self.current_position_entities.push(id);
        }
    }

    pub fn despawn(
        &mut self,
        _camera_system: &CameraSystem,
        commands: &mut Commands,
    ) {
        for entity in &self.entities {
            commands.entity(*entity).despawn_recursive();
        }
        self.entities.clear();
        for entity in &self.current_position_entities {
            commands.entity(*entity).despawn_recursive();
        }
        self.current_position_entities.clear();
    }
}
