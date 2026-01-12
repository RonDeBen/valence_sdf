pub mod debug;
pub mod forces;

use bevy::prelude::*;

// Re-export force systems for easy access
pub use forces::{apply_edge_spring_forces, apply_node_repulsion};

pub mod presets {
    /// Gentle wobbly blobs
    /// Tweak damping (0.85-0.95): higher = slower decay, longer motion
    pub const GENTLE: PhysicsPreset = PhysicsPreset {
        damping: 0.88,            // Slightly more damping (was 0.90)
        spring_stiffness: 5.0,    // Gentler pull back to rest (was 8.0)
        push_strength: 0.15,      // Reduced for smoother click response (was 0.2)
        edge_spring: 2.0,         // Softer rubber bands (was 3.0)
        repulsion_strength: 0.08, // Gentler wave propagation (was 0.15)
        repulsion_range: 2.0,     // Farther reach (unchanged)
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

/// Core physics simulation system (integration loop)
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
