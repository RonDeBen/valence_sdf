use bevy::prelude::*;

use crate::{
    game::session::PuzzleSession,
    visual::{
        nodes::{GraphNode, components::NodeVisual},
        physics::NodePhysics,
    },
};

/// System: Trigger visual effects when nodes are added to trail (click or drag-through)
pub fn trigger_trail_effects(
    session: Res<PuzzleSession>,
    mut last_trail_length: Local<usize>,
    mut nodes: Query<(&GraphNode, &mut NodePhysics, &mut NodeVisual)>,
) {
    let trail = session.current_trail();
    let current_length = trail.len();

    // Only trigger if trail length actually increased (node was ADDED, not just session mutated)
    if current_length <= *last_trail_length {
        *last_trail_length = current_length;
        return;
    }

    *last_trail_length = current_length;

    // Get the position of the last clicked node
    let last_node_pos = if let Some(&last_node) = trail.last() {
        nodes
            .iter()
            .find(|(node, _, _)| node.node_id == last_node)
            .map(|(_, physics, _)| physics.position)
    } else {
        None
    };

    let Some(_last_pos) = last_node_pos else {
        return;
    };

    // Pre-collect the previous node position if we need it for rubber band effect
    let prev_node_pos = if trail.len() > 1 {
        let prev_node_id = trail[trail.len() - 2];
        nodes
            .iter()
            .find(|(n, _, _)| n.node_id == prev_node_id)
            .map(|(_, physics, _)| physics.position)
    } else {
        None
    };

    for (graph_node, mut physics, mut visual) in &mut nodes {
        if Some(graph_node.node_id) == trail.last().copied() {
            // === TRIGGER RIPPLE on the added node ===
            visual.ripple_phase = 0.0;
            visual.ripple_amplitude = 0.8; // Full strength ripple

            // === TRIGGER GLOW on the added node ===
            visual.glow = 1.0; // Full brightness glow (immediate)

            // === RUBBER BAND SNAP: Push node away from the edge ===
            if let Some(prev_pos) = prev_node_pos {
                // Calculate direction away from previous node (along the edge)
                let edge_dir = physics.position - prev_pos;
                let distance = edge_dir.length();

                if distance > 0.01 {
                    let direction = edge_dir.normalize();
                    let snap_strength = 0.15;
                    physics.apply_impulse(direction * snap_strength);
                }
            }

            continue;
        }

        // Note: The push/impulse effect on invalid nodes has been moved to flee.rs
        // where it applies continuous flee forces while hovering
    }
}

