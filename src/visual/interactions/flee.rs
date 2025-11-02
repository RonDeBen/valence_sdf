use bevy::prelude::*;

use crate::{
    game::session::PuzzleSession,
    graph::NodeId,
    visual::{
        nodes::GraphNode,
        interactions::pointer::HoverState,
        physics::NodePhysics,
    },
};

/// Resource to track if flee mode is currently active
#[derive(Resource, Default)]
pub struct FleeMode {
    pub active: bool,
    pub trigger_node: Option<NodeId>, // Which node triggered flee mode
    pub time_active: f32,             // How long we've been fleeing
}

impl FleeMode {
    pub fn activate(&mut self, node: NodeId) {
        self.active = true;
        self.trigger_node = Some(node);
        self.time_active = 0.0;
    }

    pub fn deactivate(&mut self) {
        self.active = false;
        self.trigger_node = None;
        self.time_active = 0.0;
    }
}

/// System: Make invalid nodes flee from cursor when hovering
pub fn node_hover_flee(
    hover_state: Res<HoverState>,
    session: Res<PuzzleSession>,
    flee_mode: Res<FleeMode>,
    mut nodes: Query<(&GraphNode, &mut NodePhysics)>,
) {
    // Only apply flee forces when in active flee mode
    // Flee continues until: valid node added, or pointer released
    if !flee_mode.active {
        return;
    }

    let Some(cursor_pos) = hover_state.cursor_world_pos else {
        return;
    };

    // Get the list of nodes that should flee
    let flee_nodes: Vec<_> = session.nodes_to_flee();

    // Apply flee forces
    for (graph_node, mut physics) in &mut nodes {
        if !flee_nodes.contains(&graph_node.node_id) {
            continue;
        }

        let to_node = physics.position - cursor_pos;
        let distance = to_node.length();

        // Check if this is the node they tried to click
        let is_trigger = flee_mode.trigger_node == Some(graph_node.node_id);

        if is_trigger {
            // === DRAMATIC FLEE: The node they tried to add ===
            if distance > 0.01 && distance < 3.0 {
                let direction = to_node.normalize();
                let flee_strength = 20.0 / (distance * distance + 0.01);
                physics.apply_force(direction * flee_strength);
            }
        } else {
            // === AMBIENT FLEE: Other invalid nodes ===
            if distance > 0.01 && distance < 1.5 {
                let direction = to_node.normalize();
                let flee_strength = 5.0 / (distance * distance + 0.05);
                physics.apply_force(direction * flee_strength);
            }
        }
    }
}

/// System: Snap nodes back to rest when flee mode ends
pub fn snap_back_from_flee(
    flee_mode: Res<FleeMode>,
    mut last_flee_state: Local<bool>,
    session: Res<PuzzleSession>,
    mut nodes: Query<(&GraphNode, &mut NodePhysics)>,
) {
    // Detect flee mode just ended
    let just_deactivated = *last_flee_state && !flee_mode.active;
    *last_flee_state = flee_mode.active;

    if !just_deactivated {
        return;
    }

    info!("Flee mode ended - snapping nodes back to rest");

    let flee_nodes: Vec<_> = session.nodes_to_flee();

    // Snap all flee nodes back - INSTANT position reset, not impulse
    for (graph_node, mut physics) in &mut nodes {
        if flee_nodes.contains(&graph_node.node_id) {
            let to_rest = physics.rest_position - physics.position;
            let distance = to_rest.length();

            if distance > 0.01 {
                // Check if this is the trigger node (fled the farthest)
                let is_trigger = flee_mode.trigger_node == Some(graph_node.node_id);

                // === INSTANT SNAP - directly set position ===
                // Move most of the way back instantly
                let snap_ratio = if is_trigger { 0.95 } else { 0.90 }; // Snap 90-95% of the way
                physics.position += to_rest * snap_ratio;

                // Zero out velocity completely to prevent drift
                physics.velocity = Vec3::ZERO;

                info!(
                    "Instantly snapped node {} back {:.0}% (distance was: {:.2}, trigger: {})",
                    graph_node.node_id.0,
                    snap_ratio * 100.0,
                    distance,
                    is_trigger
                );
            }
        }
    }
}

