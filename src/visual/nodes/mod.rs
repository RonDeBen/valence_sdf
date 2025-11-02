pub mod animations;
pub mod components;

use bevy::prelude::*;
use crate::graph::NodeId;

// Re-export main types
pub use animations::update_node_visuals;
pub use components::NodeVisual;

#[derive(Component)]
pub struct GraphNode {
    pub node_id: NodeId,
}

pub fn valence_to_color(valence: usize) -> Vec4 {
    match valence {
        0 => Vec4::new(0.3, 0.3, 0.3, 1.0), // Gray
        1 => Vec4::new(0.2, 0.8, 0.2, 1.0), // Green
        2 => Vec4::new(0.2, 0.6, 1.0, 1.0), // Blue
        3 => Vec4::new(0.8, 0.8, 0.2, 1.0), // Yellow
        4 => Vec4::new(1.0, 0.6, 0.2, 1.0), // Orange
        5 => Vec4::new(1.0, 0.4, 0.4, 1.0), // Light red
        8 => Vec4::new(1.0, 0.2, 0.8, 1.0), // Magenta
        _ => Vec4::new(1.0, 0.2, 0.2, 1.0), // Red
    }
}

