use crate::visual::{
    nodes::GraphNode,
    physics::{NodePhysics, PHYSICS},
    setup::SceneMetrics,
};
use bevy::prelude::*;

pub fn apply_node_repulsion(
    scene_metrics: Res<SceneMetrics>,
    mut nodes: Query<(&GraphNode, &mut NodePhysics)>,
) {
    // 🎯 SCALE FORCES BY SCENE METRICS
    // Repulsion forces scale with grid spacing for consistency across resolutions
    let scale = scene_metrics.spacing;
    let repulsion_strength = PHYSICS.repulsion_strength * scale;
    let repulsion_range = PHYSICS.repulsion_range * scale;

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

            if distance < repulsion_range && distance > scale * 0.01 {
                // Inverse square law, but clamped
                let force_magnitude = repulsion_strength / (distance * distance);
                let max_force = repulsion_strength * 10.0; // Cap at 10x base strength
                let force = diff.normalize() * force_magnitude.min(max_force);
                physics_a.apply_force(force);
            }
        }
    }
}
