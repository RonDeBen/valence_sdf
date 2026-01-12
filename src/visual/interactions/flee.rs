use bevy::prelude::*;

use crate::{
    game::session::PuzzleSession,
    graph::NodeId,
    visual::{
        nodes::GraphNode,
        interactions::pointer::HoverState,
        physics::NodePhysics,
        setup::SceneMetrics,
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

    /// Update which node gets dramatic flee (called as cursor moves over invalid nodes)
    pub fn update_trigger(&mut self, node: NodeId) {
        self.trigger_node = Some(node);
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
    scene_metrics: Res<SceneMetrics>,
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

    // 🎯 SCALE FORCES BY SCENE METRICS
    // All distances and forces scale with grid spacing so they feel consistent
    // regardless of screen size or resolution
    let scale = scene_metrics.spacing;
    
    // Scale ranges and forces relative to grid spacing
    // Dramatic flee affects ~2.67 grid spacings
    let dramatic_range = scale * 2.67;
    let dramatic_strength = scale * 8.0;  // Reduced from 20.0
    let dramatic_min_offset = scale * 0.01;
    
    // Ambient flee affects ~1.33 grid spacings  
    let ambient_range = scale * 1.33;
    let ambient_strength = scale * 2.0;  // Reduced from 5.0
    let ambient_min_offset = scale * 0.05;
    
    let min_distance = scale * 0.01;

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
            if distance > min_distance && distance < dramatic_range {
                let direction = to_node.normalize();
                let flee_strength = dramatic_strength / (distance * distance + dramatic_min_offset);
                physics.apply_force(direction * flee_strength);
            }
        } else {
            // === AMBIENT FLEE: Other invalid nodes ===
            if distance > min_distance && distance < ambient_range {
                let direction = to_node.normalize();
                let flee_strength = ambient_strength / (distance * distance + ambient_min_offset);
                physics.apply_force(direction * flee_strength);
            }
        }
    }
}

/// System: Update flee target based on cursor hover (runs every frame during flee)
pub fn update_flee_target(
    hover_state: Res<HoverState>,
    session: Res<PuzzleSession>,
    mut flee_mode: ResMut<FleeMode>,
) {
    // Only update if flee mode is active
    if !flee_mode.active {
        return;
    }

    let Some(hovered_node_id) = hover_state.hovered_node else {
        return;
    };

    // Check if the hovered node should flee
    let flee_nodes = session.nodes_to_flee();
    if flee_nodes.contains(&hovered_node_id) {
        // Update dramatic flee target to whatever they're hovering
        flee_mode.update_trigger(hovered_node_id);
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
            }
        }
    }
}

