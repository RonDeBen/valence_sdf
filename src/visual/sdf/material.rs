use bevy::pbr::{Material, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

use crate::visual::sdf::nodes::ellipsoid::SdfSphere;
use crate::visual::sdf::edges::cylinder::SdfCylinder;

pub struct SdfMaterialPlugin;

impl Plugin for SdfMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<SdfSceneMaterial>::default());
    }
}

/// All scene data in one uniform (with proper alignment)
#[derive(ShaderType, Debug, Clone, Default)]
pub struct SdfSceneUniform {
    pub num_spheres: u32,
    pub num_cylinders: u32,
    pub _padding1: u32,
    pub _padding2: u32,
    pub spheres: [SdfSphere; 9],
    pub cylinders: [SdfCylinder; 17],
}

/// Material for the entire SDF scene
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct SdfSceneMaterial {
    #[uniform(0)]
    pub data: SdfSceneUniform,
}

impl Material for SdfSceneMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/sdf_scene.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

/// Resource to store the handle to the scene material
#[derive(Resource)]
pub struct SceneMaterialHandle(pub Handle<SdfSceneMaterial>);
