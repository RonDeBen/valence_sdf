use bevy::prelude::*;

/// Visual animation state for a node
#[derive(Component, Debug)]
pub struct NodeVisual {
    /// How much the node is squeezed (0.0 = normal, 1.0 = max squeeze)
    pub squeeze_factor: f32,
    /// Phase for ripple animation (advances over time)
    pub ripple_phase: f32,
    /// Ripple amplitude (how strong the ripple is)
    pub ripple_amplitude: f32,
    /// Target squeeze factor (for smooth interpolation)
    pub target_squeeze: f32,
    
    /// Current display color (smoothly transitions when valence changes)
    pub current_color: Vec4,
    
    /// Glow intensity (0.0 = none, 1.0 = full glow) - multi-purpose effect
    pub glow: f32,
}

impl Default for NodeVisual {
    fn default() -> Self {
        NodeVisual {
            squeeze_factor: 0.0,
            ripple_phase: 0.0,
            ripple_amplitude: 0.0,
            target_squeeze: 0.0,
            current_color: Vec4::new(0.5, 0.5, 0.5, 1.0),
            glow: 0.0,
        }
    }
}

