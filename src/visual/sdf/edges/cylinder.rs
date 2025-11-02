use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;

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
    pub wave_phase: f32,     // Where the wave is (0-1), -1 = no wave
    pub wave_amplitude: f32, // Strength of squeeze
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
            wave_phase: -1.0, // No wave by default
            wave_amplitude: 0.0,
        }
    }
}

