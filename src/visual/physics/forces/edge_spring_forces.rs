use bevy::prelude::*;

use crate::{
    game::session::PuzzleSession,
    graph::NodeId,
    visual::{
        nodes::GraphNode,
        physics::{NodePhysics, PHYSICS},
    },
};

/// Spring forces between connected nodes (rubber band effect)
pub fn apply_edge_spring_forces(
    session: Res<PuzzleSession>,
    mut nodes: Query<(&GraphNode, &mut NodePhysics)>,
) {
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
