pub mod animations;
pub mod components;

use crate::graph::NodeId;
use bevy::prelude::*;

pub use animations::update_node_visuals;
pub use components::NodeVisual;

#[derive(Component)]
pub struct GraphNode {
    pub node_id: NodeId,
}

pub fn valence_to_color(valence: usize) -> Vec4 {
    match valence {
        0 => Vec4::new(0.25, 0.25, 0.28, 1.0), // Gray (perfect as-is)

        // Slightly MORE saturated versions:
        1 => Vec4::new(0.15, 1.0, 0.30, 1.0), // GREEN (was 0.95, now 1.0)
        2 => Vec4::new(1.0, 0.95, 0.15, 1.0), // YELLOW (slightly brighter)
        3 => Vec4::new(0.20, 0.55, 1.0, 1.0), // BLUE (slightly deeper)
        4 => Vec4::new(1.0, 0.10, 0.10, 1.0), // RED (more saturated)
        5 => Vec4::new(0.90, 0.25, 0.95, 1.0), // MAGENTA (more saturated)

        6 => Vec4::new(1.0, 1.0, 1.0, 1.0),   // WHITE
        7 => Vec4::new(1.0, 0.60, 0.20, 1.0), // ORANGE
        8 => Vec4::new(0.60, 0.40, 1.0, 1.0), // PURPLE
        _ => panic!("Invalid valence: {}", valence),
    }
}
