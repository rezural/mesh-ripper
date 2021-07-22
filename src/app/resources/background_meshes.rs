use std::collections::HashMap;

use bevy::prelude::*;

use super::asset_load_checker::LoadingSource;

pub struct BackgroundMeshes {
    loading: Vec<HandleUntyped>,
    loaded: Vec<Handle<Mesh>>,
    displayed: Vec<Handle<Mesh>>,
    m2mat: HashMap<Handle<Mesh>, Color>,
    default_material_color: Color,
}

impl BackgroundMeshes {
    pub fn spawn(
        &mut self,
        commands: &mut Commands,
        materials: &mut Assets<StandardMaterial>,
    ) {
        let to_display = self.available_but_not_displayed_yet();
        println!("to_display: {}", to_display.clone().len());
        for mesh in &to_display {
            println!("spawning");
            let color = *self
                .m2mat
                .entry(mesh.clone())
                .or_insert(self.default_material_color);
            let material = materials.add(color.into());
            commands.spawn_bundle(PbrBundle {
                mesh: mesh.clone(),
                material: material,
                ..Default::default()
            });
        }
        self.displayed.extend(to_display);
    }

    fn available_but_not_displayed_yet(&self) -> Vec<Handle<Mesh>> {
        println!(
            "loaded: {}, displayed: {}",
            self.loaded.len(),
            self.displayed.len()
        );
        self.loaded
            .iter()
            .filter(|&l| !self.displayed.iter().find(|&d| l == d).is_some())
            .cloned()
            .collect()
    }
}

impl LoadingSource<Mesh> for BackgroundMeshes {
    fn loading_mut(&mut self) -> &mut Vec<HandleUntyped> {
        &mut self.loading
    }

    fn loaded_mut(&mut self) -> &mut Vec<Handle<Mesh>> {
        &mut self.loaded
    }
}

impl Default for BackgroundMeshes {
    fn default() -> Self {
        Self {
            loading: Vec::new(),
            loaded: Vec::new(),
            displayed: Vec::new(),
            m2mat: HashMap::new(),
            default_material_color: Color::rgb(81. / 255., 41. / 255., 0.),
        }
    }
}
