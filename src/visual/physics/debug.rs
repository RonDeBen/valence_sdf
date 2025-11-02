use bevy::prelude::*;

use crate::visual::{nodes::GraphNode, physics::NodePhysics};

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
