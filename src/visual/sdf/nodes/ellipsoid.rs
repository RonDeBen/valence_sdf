use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;

/// A single SDF sphere in the scene with infection animation
#[derive(ShaderType, Debug, Clone, Copy)]
pub struct SdfSphere {
    pub center: Vec3,
    pub radius: f32,

    // Color infection system
    pub base_color: Vec4,        // Color before infection
    pub target_color: Vec4,      // Color after infection
    pub infection_progress: f32, // 0.0 = just started, 1.0 = complete
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
            infection_progress: 1.0, // Start fully infected (no animation initially)
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

