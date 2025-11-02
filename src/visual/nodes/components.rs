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
    /// Is this node invalid/spiky?
    pub is_invalid: bool,
    /// Base radius (for scaling effects)
    pub base_radius: f32,

    // Infection-based color transition
    /// Base color (before infection starts)
    pub base_color: Vec4,
    /// Current color (for display during infection)
    pub current_color: Vec4,
    /// Target color (after infection completes)
    pub target_color: Vec4,
    /// Infection progress (0.0 = just started, 1.0 = complete)
    pub infection_progress: f32,
    
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
            is_invalid: false,
            base_radius: 1.0,
            base_color: Vec4::new(0.5, 0.5, 0.5, 1.0),
            current_color: Vec4::new(0.5, 0.5, 0.5, 1.0),
            target_color: Vec4::new(0.5, 0.5, 0.5, 1.0),
            infection_progress: 1.0, // Start fully infected (instant color at spawn)
            glow: 0.0,
        }
    }
}

