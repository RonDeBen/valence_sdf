use bevy::prelude::*;

use crate::{
    game::session::PuzzleSession,
    visual::{
        graph::GraphNode,
        physics::{NodePhysics, NodeVisual, PHYSICS},
    },
};
/// Trigger effects when user interacts with nodes
pub fn trigger_node_interactions(
    session: Res<PuzzleSession>,
    mut nodes: Query<(&GraphNode, &mut NodePhysics, &mut NodeVisual)>,
) {
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

    // Get nodes that should flee (invalid to add)
    let flee_nodes: Vec<_> = session.nodes_to_flee();

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
            // === TRIGGER RIPPLE on the clicked node ===
            visual.ripple_phase = 0.0;
            visual.ripple_amplitude = 1.0; // Full strength ripple
            
            // === TRIGGER GLOW on the clicked node ===
            visual.glow = 1.0; // Full brightness glow (immediate)

            // === RUBBER BAND SNAP: Push node away from the edge ===
            if let Some(prev_pos) = prev_node_pos {
                // Calculate direction away from previous node (along the edge)
                let edge_dir = physics.position - prev_pos;
                let distance = edge_dir.length();

                if distance > 0.01 {
                    let direction = edge_dir.normalize();
                    let snap_strength = 0.5;
                    physics.apply_impulse(direction * snap_strength);
                }
            }
            continue;
        }

        // Only push invalid nodes
        if !flee_nodes.contains(&graph_node.node_id) {
            continue;
        }

        // Calculate direction away from clicked node
        let to_other = physics.position - last_pos;
        let distance = to_other.length();

        if distance > 0.01 && distance < 2.5 {
            let direction = to_other.normalize();
            let push_strength = PHYSICS.push_strength / (distance + 0.5);
            physics.apply_impulse(direction * push_strength);
        }
    }
}
