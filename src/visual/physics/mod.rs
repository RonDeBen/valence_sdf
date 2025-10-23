pub mod debug;
pub mod edge_spring_forces;
pub mod node_interactions;
pub mod node_repulsion;

use bevy::prelude::*;

// Re-export the resource for initialization
pub use node_interactions::LastTrailLength;

pub mod presets {
    /// Gentle wobbly blobs
    /// Tweak damping (0.85-0.95): higher = slower decay, longer motion
    pub const GENTLE: PhysicsPreset = PhysicsPreset {
        damping: 0.90,            // Higher = motion lasts longer
        spring_stiffness: 8.0,    // Gentler pull back to rest
        push_strength: 0.2,       // Reduced for smoother click response
        edge_spring: 3.0,         // Softer rubber bands (reduced from 5.0)
        repulsion_strength: 0.15, // Stronger wave propagation (was 0.05)
        repulsion_range: 2.0,     // Farther reach (was 1.5)
    };

    /// Bouncy energetic movement
    pub const BOUNCY: PhysicsPreset = PhysicsPreset {
        damping: 0.95,
        spring_stiffness: 20.0,
        push_strength: 0.8,
        edge_spring: 15.0,
        repulsion_strength: 0.3,
        repulsion_range: 1.5,
    };

    /// Sluggish heavy movement
    pub const SLUGGISH: PhysicsPreset = PhysicsPreset {
        damping: 0.7,
        spring_stiffness: 5.0,
        push_strength: 0.1,
        edge_spring: 2.0,
        repulsion_strength: 0.05,
        repulsion_range: 1.2,
    };

    #[derive(Debug, Clone, Copy)]
    pub struct PhysicsPreset {
        pub damping: f32,
        pub spring_stiffness: f32,
        pub push_strength: f32,
        pub edge_spring: f32,
        pub repulsion_strength: f32,
        pub repulsion_range: f32,
    }
}

// Current active preset
const PHYSICS: presets::PhysicsPreset = presets::GENTLE;

/// Physics state for a node
#[derive(Component, Debug)]
pub struct NodePhysics {
    /// Current position (separate from Transform - this is the SDF center)
    pub position: Vec3,
    /// Current velocity
    pub velocity: Vec3,
    /// Accumulated forces this frame
    pub forces: Vec3,
    /// Mass (affects acceleration)
    pub mass: f32,
    /// Damping factor (0.0 = full damping, 1.0 = no damping)
    pub damping: f32,
    /// Rest position (where the node wants to be)
    pub rest_position: Vec3,
    /// Spring stiffness back to rest position
    pub spring_stiffness: f32,
}

impl Default for NodePhysics {
    fn default() -> Self {
        NodePhysics {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            forces: Vec3::ZERO,
            mass: 1.0,
            damping: PHYSICS.damping,
            rest_position: Vec3::ZERO,
            spring_stiffness: PHYSICS.spring_stiffness,
        }
    }
}

impl NodePhysics {
    /// Apply a force to this node
    pub fn apply_force(&mut self, force: Vec3) {
        self.forces += force;
    }

    /// Apply an impulse (instant velocity change)
    pub fn apply_impulse(&mut self, impulse: Vec3) {
        self.velocity += impulse / self.mass;
    }
}

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

/// Physics simulation system
pub fn simulate_node_physics(time: Res<Time>, mut nodes: Query<&mut NodePhysics>) {
    let dt = time.delta_secs();

    for mut physics in &mut nodes {
        // Spring force back to rest position (Hooke's law: F = -kx)
        let displacement = physics.position - physics.rest_position;
        let spring_force = -displacement * physics.spring_stiffness;
        physics.apply_force(spring_force);

        // Calculate acceleration: F = ma → a = F/m
        let acceleration = physics.forces / physics.mass;

        // Update velocity (Euler integration)
        physics.velocity += acceleration * dt;

        // Apply damping (exponential decay)
        let damping = physics.damping;
        physics.velocity *= damping;

        // Update position
        let velocity = physics.velocity;
        physics.position += velocity * dt;

        // Clear forces for next frame
        physics.forces = Vec3::ZERO;
    }
}
