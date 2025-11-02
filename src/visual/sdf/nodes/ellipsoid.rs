use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;

/// A single SDF sphere in the scene
#[derive(ShaderType, Debug, Clone, Copy)]
pub struct SdfSphere {
    pub center: Vec3,
    pub radius: f32,

    /// Display color (pre-computed on CPU)
    pub color: Vec4,

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
            color: Vec4::ONE,
            stretch_direction: Vec3::Y,
            stretch_factor: 1.0,
            ripple_phase: 0.0,
            ripple_amplitude: 0.0,
            spike_amount: 0.0,
            _padding: 0.0,
        }
    }
}

