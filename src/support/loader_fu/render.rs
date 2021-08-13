use std::collections::HashMap;

use bevy::{
    ecs::system::EntityCommands,
    prelude::{shape::Icosphere, *},
    render::{
        mesh::{Indices, VertexAttributeValues},
        pipeline::PrimitiveTopology,
    },
};
use bevy_inspector_egui::Inspectable;
use nalgebra::{Point3, Quaternion, Vector3};
use parry3d::math::Real;
use rand::{prelude::IteratorRandom, thread_rng};
use rapier3d::prelude::Cone;

use super::features::Features;

use serde::*;

#[derive(Eq, PartialEq, Hash, Debug, Inspectable, Serialize, Deserialize, Copy, Clone)]
pub enum PointRenderOptions {
    Sphere,
    Directional,
}

pub struct RenderCache {
    pub radius: f32,
    pub cache: HashMap<PointRenderOptions, Handle<Mesh>>,
}

impl RenderCache {
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            cache: HashMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        key: PointRenderOptions,
        value: Handle<Mesh>,
    ) -> Option<Handle<Mesh>> {
        self.cache.insert(key, value)
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }

    pub fn get(
        &self,
        key: &PointRenderOptions,
    ) -> Option<&Handle<Mesh>> {
        self.cache.get(key)
    }
}

pub struct FeatureAwareRenderer {
    handle: Handle<Mesh>,
}

/// This handles rendering 'meshes', that are either:
/// Meshes, that have vertices, and indices, and normals hopefully, these are rendered as-is
/// Particles, that have vertices, and possibly normals, but not indices (i.e. no triangles)
/// Particles are rendered either via spheres, or directional arrows
impl FeatureAwareRenderer {
    pub fn new(mesh: Handle<Mesh>) -> Self {
        Self { handle: mesh }
    }

    pub fn cache_meshes(
        meshes: &mut Assets<Mesh>,
        radius: f32,
        render_cache: &mut RenderCache,
    ) {
        render_cache.clear();
        render_cache.radius = radius;
        let mr = MeshRenderer;
        mr.cache_meshes(render_cache, meshes, radius);

        let pr = PointsRenderer;
        pr.cache_meshes(render_cache, meshes, radius);
    }

    pub fn spawn(
        &self,
        commands: &mut Commands,
        meshes: &Assets<Mesh>,
        material: Handle<StandardMaterial>,
        render_options: PointRenderOptions,
        cache: &RenderCache,
        sampled_indices: &Vec<usize>,
    ) -> Vec<Entity> {
        let mesh = meshes.get(self.handle.clone());
        if let Some(mesh) = mesh {
            let renderer = Self::renderer(mesh);
            let features = Features::new(mesh);
            return renderer.spawn(
                commands,
                self.handle.clone(),
                material,
                features,
                render_options,
                cache,
                sampled_indices,
            );
        }

        Vec::new()
    }

    pub fn sample_indices(
        mesh: &Mesh,
        sample_size: usize,
    ) -> Vec<usize> {
        let features = Features::new(mesh);
        let mut rng = rand::thread_rng();

        if let Some(vertices) = features.vertices() {
            vertices
                .iter()
                .enumerate()
                .into_iter()
                .map(|(idx, _)| idx)
                .choose_multiple(&mut rng, sample_size)
        } else {
            Vec::new()
        }
    }

    pub fn despawn(
        &self,
        commands: &mut Commands,
        meshes: &Assets<Mesh>,
        entities: Vec<Entity>,
    ) {
        let mesh = meshes.get(self.handle.clone());
        if let Some(mesh) = mesh {
            let renderer = Self::renderer(mesh);
            renderer.despawn(commands, entities);
        }
    }

    fn renderer(mesh: &Mesh) -> &dyn Renderer {
        let features = Features::new(mesh);
        // This is a mesh
        let renderer: &dyn Renderer = if features.has_indices() {
            &MeshRenderer as &dyn Renderer
        } else {
            &PointsRenderer as &dyn Renderer
        };

        renderer
    }
}

trait Renderer {
    fn spawn(
        &self,
        commands: &mut Commands,
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        features: Features,
        render_options: PointRenderOptions,
        cache: &RenderCache,
        sampled_indices: &Vec<usize>,
    ) -> Vec<Entity>;

    fn despawn(
        &self,
        commands: &mut Commands,
        entities: Vec<Entity>,
    ) {
        for entity in entities {
            let mut entity = commands.entity(entity);
            entity.despawn_recursive();
        }
    }

    fn cache_meshes(
        &self,
        _cache: &mut RenderCache,
        _meshes: &mut Assets<Mesh>,
        _radius: f32,
    ) {
    }
}

pub struct MeshRenderer;

impl Renderer for MeshRenderer {
    fn spawn(
        &self,
        commands: &mut Commands,
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        _features: Features,
        _render_options: PointRenderOptions,
        _cache: &RenderCache,
        sampled_indices: &Vec<usize>,
    ) -> Vec<Entity> {
        let entity = commands
            .spawn()
            .insert_bundle(PbrBundle {
                mesh: mesh.clone(),
                material: material.clone(),
                ..Default::default()
            })
            .id();
        vec![entity]
    }
}

pub struct PointsRenderer;

impl PointsRenderer {
    fn spawn_sphere(
        &self,
        commands: &mut Commands,
        cache: &RenderCache,
        material: Handle<StandardMaterial>,
        origin: Vec3,
        normal: Option<Vec3>,
    ) -> Option<Entity> {
        // println!("sphere spawn");
        let po = PointRenderOptions::Sphere;
        if let Some(mesh) = cache.get(&po) {
            Some(
                commands
                    .spawn_bundle(PbrBundle {
                        mesh: mesh.clone(),
                        material: material.clone(),
                        transform: Self::transform(origin, normal),
                        ..Default::default()
                    })
                    .id(),
            )
        } else {
            None
        }
    }

    fn spawn_directional(
        &self,
        commands: &mut Commands,
        cache: &RenderCache,
        material: Handle<StandardMaterial>,
        origin: Vec3,
        normal: Option<Vec3>,
    ) -> Option<Entity> {
        let po = PointRenderOptions::Directional;
        if let Some(mesh) = cache.get(&po) {
            Some(
                commands
                    .spawn_bundle(PbrBundle {
                        mesh: mesh.clone(),
                        material: material.clone(),
                        transform: Self::transform(origin, normal),
                        ..Default::default()
                    })
                    .id(),
            )
        } else {
            None
        }
    }

    fn transform(
        origin: Vec3,
        normal: Option<Vec3>,
    ) -> Transform {
        if let Some(normal) = normal {
            // println!("got normal in transform");
            let cone_paxis: Quaternion<Real> =
                Quaternion::from_vector(-Vector3::y().to_homogeneous());
            // println!("normal: {:?}", normal);

            let n_normal: Vector3<Real> = normal.into();
            // println!("n_normal: {:?}", n_normal);
            let vr = Quaternion::from_vector(n_normal.normalize().to_homogeneous());
            // println!("vr: {:?}", vr);

            let rotation = (vr - cone_paxis).normalize();

            let rotation = Quat::from_xyzw(rotation.i, rotation.j, rotation.k, rotation.w);

            // println!("bevy_rotation: {:?}", rotation);

            let transform = Transform {
                translation: origin,
                rotation,
                ..Default::default()
            };

            // println!("{:?}", transform);

            return transform;
        }
        Transform {
            translation: origin,
            ..Default::default()
        }
    }

    pub fn cache_meshes(
        &self,
        cache: &mut RenderCache,
        meshes: &mut Assets<Mesh>,
        radius: f32,
    ) {
        let sphere = Icosphere {
            radius,
            subdivisions: 8,
        };
        cache.insert(PointRenderOptions::Sphere, meshes.add(Mesh::from(sphere)));

        let arrow = Self::gen_arrow_mesh(radius);
        cache.insert(PointRenderOptions::Directional, meshes.add(arrow));
    }

    pub fn gen_arrow_mesh(radius: f32) -> Mesh {
        let cone = Cone::new(radius / 2., radius / 4.);
        Self::bevy_mesh(cone.to_trimesh(10))
    }

    /// cadged from https://github.com/dimforge/rapier/blob/0bb2f08deafe69afcb514728b584c590b3559fd2/src_testbed%2Fobjects%2Fnode.rs#L280
    fn bevy_mesh(buffers: (Vec<Point3<f32>>, Vec<[u32; 3]>)) -> Mesh {
        let (vtx, idx) = buffers;
        let mut normals: Vec<[f32; 3]> = vec![];
        let mut vertices: Vec<[f32; 3]> = vec![];

        for idx in idx {
            let a = vtx[idx[0] as usize];
            let b = vtx[idx[1] as usize];
            let c = vtx[idx[2] as usize];

            vertices.push(a.into());
            vertices.push(b.into());
            vertices.push(c.into());
        }

        for vtx in vertices.chunks(3) {
            let a = Point3::from(vtx[0]);
            let b = Point3::from(vtx[1]);
            let c = Point3::from(vtx[2]);
            let n = (b - a).cross(&(c - a)).normalize();
            normals.push(n.into());
            normals.push(n.into());
            normals.push(n.into());
        }

        normals
            .iter_mut()
            .for_each(|n| *n = Vector3::from(*n).normalize().into());
        let indices: Vec<_> = (0..vertices.len() as u32).collect();
        let uvs: Vec<_> = (0..vertices.len()).map(|_| [0.0, 0.0]).collect();

        // Generate the mesh
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_attribute(
            Mesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::from(vertices),
        );
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, VertexAttributeValues::from(normals));
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::from(uvs));
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh
    }
}

impl Renderer for PointsRenderer {
    fn spawn(
        &self,
        commands: &mut Commands,
        _mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        features: Features,
        render_options: PointRenderOptions,
        cache: &RenderCache,
        sampled_indices: &Vec<usize>,
    ) -> Vec<Entity> {
        let mut entities = Vec::new();
        if let Some(vertices) = features.vertices() {
            let normals = features.normals();

            for &idx in sampled_indices {
                let vertex = vertices[idx];
                let origin = Vec3::new(vertex[0], vertex[1], vertex[2]);

                let normal = if let Some(normals) = normals {
                    if let Some(normal) = normals.get(idx) {
                        let normal = Vec3::new(normal[0], normal[1], normal[2]);
                        Some(normal)
                    } else {
                        None
                    }
                } else {
                    None
                };

                let entity = match render_options {
                    PointRenderOptions::Sphere => {
                        self.spawn_sphere(commands, &cache, material.clone(), origin, normal)
                    }
                    PointRenderOptions::Directional => {
                        self.spawn_directional(commands, &cache, material.clone(), origin, normal)
                    }
                };

                if let Some(entity) = entity {
                    entities.push(entity);
                }
            }
        }
        entities
    }
}
