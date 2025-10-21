use crate::visual::{
    graph::GraphNode,
    physics::{NodePhysics, PHYSICS},
};
use bevy::prelude::*;

pub fn apply_node_repulsion(mut nodes: Query<(&GraphNode, &mut NodePhysics)>) {
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
