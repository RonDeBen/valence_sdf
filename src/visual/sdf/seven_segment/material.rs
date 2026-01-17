//! Seven-segment display material with animated digit transitions.
//!
//! This module provides the material and shader for rendering 7-segment style digits
//! with fancy blob transition animations (splitting, flying, morphing).
//!
//! Uses a unified rendering approach: one large plane renders all HUD elements
//! from an array of HudInstance structs.

use bevy::pbr::{Material, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

/// Plugin that registers the SevenSegmentMaterial for use in the HUD
pub struct SevenSegmentMaterialPlugin;

impl Plugin for SevenSegmentMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<SevenSegmentMaterial>::default());
    }
}

/// Maximum number of HUD instances (digits + slashes)
pub const MAX_HUD_INSTANCES: usize = 12;

/// A single HUD element instance (digit or slash)
#[derive(Clone, Copy, Debug, ShaderType)]
#[repr(C)]
pub struct HudInstance {
    /// Element kind: 0 = digit, 1 = slash
    pub kind: u32,
    /// Current 7-segment bitmask (target for transitions)
    pub mask: u32,
    /// Previous mask (for animated transitions, from_mask → mask)
    pub from_mask: u32,
    /// Transition progress: 0.0 = showing from_mask, 1.0 = showing mask
    pub transition_progress: f32,
    /// Position in world XY space: vec2(world_x, world_y)
    pub pos: Vec2,
    /// Scale multiplier for the element
    pub scale: f32,
    /// Padding to reach 48 bytes (next multiple of 16 from 40)
    pub _pad1: u32,
    pub _pad2: u32,
    pub _pad3: u32,
    pub _pad4: u32,
}

impl Default for HudInstance {
    fn default() -> Self {
        Self {
            kind: 0,
            mask: 0,
            from_mask: 0,
            transition_progress: 1.0, // Default to "transition complete"
            pos: Vec2::ZERO,
            scale: 0.0,
            _pad1: 0,
            _pad2: 0,
            _pad3: 0,
            _pad4: 0,
        }
    }
}

#[derive(ShaderType, Debug, Clone)]
pub struct SevenSegmentData {
    pub time: f32,
    pub hud_count: u32,
    pub _padding1: u32,
    pub _padding2: u32,

    pub hud: [HudInstance; MAX_HUD_INSTANCES],
}

impl Default for SevenSegmentData {
    fn default() -> Self {
        Self {
            time: 0.0,
            hud_count: 0,
            _padding1: 0,
            _padding2: 0,
            hud: [HudInstance::default(); MAX_HUD_INSTANCES],
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct SevenSegmentMaterial {
    #[uniform(0)]
    pub data: SevenSegmentData,
}

impl Material for SevenSegmentMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/seven_segment.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
