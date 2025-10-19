use bevy::pbr::{Material, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

pub struct SdfMaterialPlugin;
impl Plugin for SdfMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<SdfSphereMaterial>::default());
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[bind_group_data(SdfSphereMaterialKey)]
pub struct SdfSphereMaterial {
    #[uniform(0)]
    pub data: SdfSphereMaterialData,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SdfSphereMaterialKey;

impl From<&SdfSphereMaterial> for SdfSphereMaterialKey {
    fn from(_: &SdfSphereMaterial) -> Self {
        SdfSphereMaterialKey
    }
}

// This struct must match the WGSL layout exactly
#[derive(ShaderType, Debug, Clone, Copy)]
pub struct SdfSphereMaterialData {
    pub color: Vec4,
    pub center: Vec3,
    pub radius: f32,
}

impl Material for SdfSphereMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/sdf_sphere.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
    fn depth_bias(&self) -> f32 {
        0.0
    }
}
