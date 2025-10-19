use crate::{game::session::PuzzleSession, graph::NodeId, visual::graph::GraphNode};
use bevy::prelude::*;

// ============================================================================
// EASING FUNCTIONS for smooth animations
// ============================================================================

/// Ease-in-out cubic: slow at start and end, fast in the middle
fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// Ease-in quadratic: slow at start, accelerates
#[allow(dead_code)]
fn ease_in_quad(t: f32) -> f32 {
    t * t
}

/// Ease-out quadratic: fast at start, decelerates
#[allow(dead_code)]
fn ease_out_quad(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

/// Ease-out cubic: fast at start, decelerates at end
fn ease_out_cubic(t: f32) -> f32 {
    let x = 1.0 - t;
    1.0 - x * x * x
}

// ============================================================================
// DEBUG FLAGS - Toggle individual systems on/off for debugging
// ============================================================================

/// Debug flags to enable/disable individual physics systems
pub mod debug_flags {
    // === Physics Systems ===
    pub const ENABLE_SPRING_TO_REST: bool = true; // Core: nodes return to grid
    pub const ENABLE_EDGE_SPRINGS: bool = true; // Rubber band connections
    pub const ENABLE_NODE_REPULSION: bool = false; // Personal space (can cause jank)
    pub const ENABLE_CLICK_IMPULSE: bool = true; // Push nodes on click

    // === Visual Effects ===
    pub const ENABLE_RIPPLES: bool = false; // Ripple waves (currently disabled)
    pub const ENABLE_VELOCITY_SQUASH: bool = true; // Motion blur effect
    pub const ENABLE_SPIKES: bool = false; // Invalid node spikes (not implemented)
}

// ============================================================================
// PHYSICS TUNING CONSTANTS
// ============================================================================

/// Preset physics configurations
pub mod presets {
    /// Gentle wobbly blobs (recommended default)
    /// Tweak damping (0.85-0.95): higher = slower decay, longer motion
    pub const GENTLE: PhysicsPreset = PhysicsPreset {
        damping: 0.90,            // Higher = motion lasts longer
        spring_stiffness: 8.0,    // Gentler pull back to rest
        push_strength: 0.2,       // Reduced for smoother click response
        edge_spring: 3.0,         // Softer rubber bands (reduced from 5.0)
        repulsion_strength: 0.05, // Reduced personal space (less jank)
        repulsion_range: 1.5,
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

// ============================================================================
// COMPONENTS
// ============================================================================

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
        }
    }
}

/// Physics simulation system
pub fn simulate_node_physics(time: Res<Time>, mut nodes: Query<&mut NodePhysics>) {
    use debug_flags::ENABLE_SPRING_TO_REST;

    let dt = time.delta_secs();

    for mut physics in &mut nodes {
        // Spring force back to rest position (Hooke's law: F = -kx)
        if ENABLE_SPRING_TO_REST {
            let displacement = physics.position - physics.rest_position;
            let spring_force = -displacement * physics.spring_stiffness;
            physics.apply_force(spring_force);
        }

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

/// Spring forces between connected nodes (rubber band effect)
pub fn apply_edge_spring_forces(
    session: Res<PuzzleSession>,
    mut nodes: Query<(&GraphNode, &mut NodePhysics)>,
) {
    use debug_flags::ENABLE_EDGE_SPRINGS;

    if !ENABLE_EDGE_SPRINGS {
        return; // Easy toggle for debugging!
    }

    let edges = session.edges();

    // Collect all node data first to avoid borrow conflicts
    let node_data: Vec<_> = nodes
        .iter()
        .map(|(node, physics)| (node.node_id, physics.position, physics.rest_position))
        .collect();

    // Calculate forces for each edge
    let mut forces: Vec<(NodeId, Vec3)> = Vec::new();

    for edge in edges.edges_in_order() {
        // Find the two nodes
        let node_a_data = node_data.iter().find(|(id, _, _)| *id == edge.from);
        let node_b_data = node_data.iter().find(|(id, _, _)| *id == edge.to);

        let Some(&(_, pos_a, rest_a)) = node_a_data else {
            continue;
        };
        let Some(&(_, pos_b, rest_b)) = node_b_data else {
            continue;
        };

        // Calculate desired rest length (distance between rest positions)
        let rest_length = (rest_b - rest_a).length();
        let current_length = (pos_b - pos_a).length();

        if current_length < 0.001 {
            continue; // Avoid division by zero
        }

        // Spring force: F = k * (current_length - rest_length)
        let direction = (pos_b - pos_a) / current_length;
        let extension = current_length - rest_length;
        let force_magnitude = PHYSICS.edge_spring * extension;

        let force = direction * force_magnitude;

        // Store forces to apply
        forces.push((edge.from, force));
        forces.push((edge.to, -force));
    }

    // Now apply all forces
    for (node_id, force) in forces {
        for (graph_node, mut physics) in &mut nodes {
            if graph_node.node_id == node_id {
                physics.apply_force(force);
                break;
            }
        }
    }
}

/// Node repulsion forces (nodes push each other away slightly)
pub fn apply_node_repulsion(mut nodes: Query<(&GraphNode, &mut NodePhysics)>) {
    use debug_flags::ENABLE_NODE_REPULSION;

    if !ENABLE_NODE_REPULSION {
        return; // Often causes jank, easy to disable!
    }

    let repulsion_strength = PHYSICS.repulsion_strength;
    let repulsion_range = PHYSICS.repulsion_range;

    // Collect positions first to avoid borrow issues
    let positions: Vec<_> = nodes
        .iter()
        .map(|(node, physics)| (node.node_id, physics.position))
        .collect();

    // Apply repulsion forces
    for (node_a, mut physics_a) in &mut nodes {
        for &(node_b_id, pos_b) in &positions {
            if node_a.node_id == node_b_id {
                continue; // Don't repel self
            }

            let diff = physics_a.position - pos_b;
            let distance = diff.length();

            if distance < repulsion_range && distance > 0.01 {
                // Inverse square law, but clamped
                let force_magnitude = repulsion_strength / (distance * distance);
                let max_force = repulsion_strength * 10.0; // Cap at 10x base strength
                let force = diff.normalize() * force_magnitude.min(max_force);
                physics_a.apply_force(force);
            }
        }
    }
}

/// Trigger effects when user interacts with nodes
pub fn trigger_node_interactions(
    session: Res<PuzzleSession>,
    mut nodes: Query<(&GraphNode, &mut NodePhysics, &mut NodeVisual)>,
) {
    use debug_flags::ENABLE_CLICK_IMPULSE;

    if !ENABLE_CLICK_IMPULSE {
        return; // Disable click effects for debugging
    }

    // Only check if session changed (new node added)
    if !session.is_changed() {
        return;
    }

    let trail = session.current_trail();

    // Get the position of the last clicked node
    let last_node_pos = if let Some(&last_node) = trail.last() {
        nodes
            .iter()
            .find(|(node, _, _)| node.node_id == last_node)
            .map(|(_, physics, _)| physics.position)
    } else {
        None
    };

    let Some(last_pos) = last_node_pos else {
        return;
    };

    // Push all OTHER nodes away from the clicked node
    for (graph_node, mut physics, mut visual) in &mut nodes {
        if Some(graph_node.node_id) == trail.last().copied() {
            // Trigger effects on the clicked node (ripple disabled for debugging)
            // visual.ripple_phase = 0.0;
            // visual.ripple_amplitude = 0.3;
            info!("Clicked node {}", graph_node.node_id.0);
            continue;
        }

        // Calculate direction away from clicked node
        let to_other = physics.position - last_pos;
        let distance = to_other.length();

        if distance > 0.01 && distance < 2.5 {
            // Only push nearby nodes
            let direction = to_other.normalize();
            let push_strength = PHYSICS.push_strength / (distance + 0.5);

            // Apply push impulse
            physics.apply_impulse(direction * push_strength);
            info!(
                "Pushing node {} away from {} (distance: {:.2})",
                graph_node.node_id.0,
                trail.last().unwrap().0,
                distance
            );
        }
    }
}

/// Update visual animation states
pub fn update_node_visuals(
    time: Res<Time>,
    session: Res<PuzzleSession>,
    mut nodes: Query<(&GraphNode, &NodePhysics, &mut NodeVisual)>,
) {
    use debug_flags::{ENABLE_RIPPLES, ENABLE_SPIKES, ENABLE_VELOCITY_SQUASH};

    let dt = time.delta_secs();
    let valences = session.current_valences();

    for (graph_node, physics, mut visual) in &mut nodes {
        let valence = valences.get(graph_node.node_id);

        // === Color Infection Animation (spreading from edge contact points) ===
        let new_target = crate::visual::graph::valence_to_color(valence);

        // If target color changed, start new infection
        if (new_target - visual.target_color).length() > 0.1 {
            visual.base_color = visual.current_color; // Save current as base
            visual.target_color = new_target;
            visual.infection_progress = 0.0; // Start infection animation
        }

        // Animate infection progress (spreading across surface)
        if visual.infection_progress < 1.0 {
            visual.infection_progress += dt * 2.5; // Speed of infection spread (higher = faster)
            visual.infection_progress = visual.infection_progress.min(1.0);
        }

        // Apply easing to make it feel snappier
        let eased_progress = ease_out_cubic(visual.infection_progress);

        // For overall display color
        visual.current_color = visual.base_color.lerp(visual.target_color, eased_progress);

        // === Squeeze from valence (always on) ===
        visual.target_squeeze = match valence {
            0 => 0.3, // Completed nodes are squeezed
            1 => 0.1, // Almost done - slight squeeze
            _ => 0.0, // Normal
        };
        visual.squeeze_factor = visual.squeeze_factor.lerp(visual.target_squeeze, dt * 2.0);

        // === Velocity squash (optional, doesn't stack with above) ===
        if ENABLE_VELOCITY_SQUASH {
            let speed = physics.velocity.length();
            // Only apply if moving fast AND not already squeezed by valence
            if speed > 0.2 && visual.target_squeeze < 0.05 {
                let velocity_squeeze = (speed * 0.05).min(0.3);
                visual.squeeze_factor = visual.squeeze_factor.max(velocity_squeeze);
            }
        }

        // === Ripples (optional) ===
        if ENABLE_RIPPLES {
            if visual.ripple_amplitude > 0.01 {
                visual.ripple_phase += dt * 8.0;
                visual.ripple_amplitude *= 0.97;
            } else {
                visual.ripple_amplitude = 0.0;
            }
        } else {
            visual.ripple_amplitude = 0.0; // Force off
        }

        // === Spikes for invalid nodes (not implemented yet) ===
        if ENABLE_SPIKES {
            // TODO: Implement invalid node detection
            visual.is_invalid = false;
        } else {
            visual.is_invalid = false;
        }
    }
}

/// Debug visualization of physics state
pub fn debug_physics(nodes: Query<(&GraphNode, &NodePhysics)>, mut gizmos: Gizmos) {
    for (_graph_node, physics) in &nodes {
        // Draw rest position (blue sphere)
        gizmos.sphere(
            Isometry3d::new(physics.rest_position, Quat::IDENTITY),
            0.1,
            Color::srgb(0.0, 0.5, 1.0),
        );

        // Draw current position (green sphere)
        gizmos.sphere(
            Isometry3d::new(physics.position, Quat::IDENTITY),
            0.15,
            Color::srgb(0.0, 1.0, 0.0),
        );

        // Draw line from rest to current
        gizmos.line(
            physics.rest_position,
            physics.position,
            Color::srgb(1.0, 1.0, 0.0),
        );

        // Draw velocity vector
        if physics.velocity.length() > 0.01 {
            gizmos.arrow(
                physics.position,
                physics.position + physics.velocity,
                Color::srgb(1.0, 0.0, 0.0),
            );
        }
    }
}
