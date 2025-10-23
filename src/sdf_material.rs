use bevy::pbr::{Material, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

pub struct SdfMaterialPlugin;

impl Plugin for SdfMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<SdfSceneMaterial>::default());
    }
}

/// A single SDF sphere in the scene with infection animation
#[derive(ShaderType, Debug, Clone, Copy)]
pub struct SdfSphere {
    pub center: Vec3,
    pub radius: f32,
    
    // Color infection system
    pub base_color: Vec4,           // Color before infection
    pub target_color: Vec4,         // Color after infection
    pub infection_progress: f32,    // 0.0 = just started, 1.0 = complete
    pub _padding1: f32,
    pub _padding2: f32,
    pub _padding3: f32,
    
    pub stretch_direction: Vec3,
    pub stretch_factor: f32,
    pub ripple_phase: f32,
    pub ripple_amplitude: f32,
    pub spike_amount: f32,
    pub _padding: f32,
}

impl Default for SdfSphere {
    fn default() -> Self {
        SdfSphere {
            center: Vec3::ZERO,
            radius: 1.0,
            base_color: Vec4::ONE,
            target_color: Vec4::ONE,
            infection_progress: 1.0,  // Start fully infected (no animation initially)
            _padding1: 0.0,
            _padding2: 0.0,
            _padding3: 0.0,
            stretch_direction: Vec3::Y,
            stretch_factor: 1.0,
            ripple_phase: 0.0,
            ripple_amplitude: 0.0,
            spike_amount: 0.0,
            _padding: 0.0,
        }
    }
}

/// A cylinder connecting two spheres (edge)
#[derive(ShaderType, Debug, Clone, Copy)]
pub struct SdfCylinder {
    pub start: Vec3,
    pub _padding1: f32,
    pub end: Vec3,
    pub radius: f32,
    pub color: Vec4,
    
    // Track which nodes this cylinder connects (for infection gradient)
    pub node_a_idx: u32,
    pub node_b_idx: u32,
    
    // Tension wave animation
    pub wave_phase: f32,      // Where the wave is (0-1), -1 = no wave
    pub wave_amplitude: f32,  // Strength of squeeze
}

impl Default for SdfCylinder {
    fn default() -> Self {
        SdfCylinder {
            start: Vec3::ZERO,
            _padding1: 0.0,
            end: Vec3::ZERO,
            radius: 0.1,
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            node_a_idx: 0,
            node_b_idx: 0,
            wave_phase: -1.0,   // No wave by default
            wave_amplitude: 0.0,
        }
    }
}

/// All scene data in one uniform (with proper alignment)
#[derive(ShaderType, Debug, Clone)]
pub struct SdfSceneUniform {
    pub num_spheres: u32,
    pub num_cylinders: u32,
    pub _padding1: u32,
    pub _padding2: u32,
    pub spheres: [SdfSphere; 9],
    pub cylinders: [SdfCylinder; 17],
}

impl Default for SdfSceneUniform {
    fn default() -> Self {
        SdfSceneUniform {
            num_spheres: 0,
            num_cylinders: 0,
            _padding1: 0,
            _padding2: 0,
            spheres: [SdfSphere::default(); 9],
            cylinders: [SdfCylinder::default(); 17],
        }
    }
}

/// Material for the entire SDF scene
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct SdfSceneMaterial {
    #[uniform(0)]
    pub data: SdfSceneUniform,
}

impl Default for SdfSceneMaterial {
    fn default() -> Self {
        SdfSceneMaterial {
            data: SdfSceneUniform::default(),
        }
    }
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
