use crate::game::{puzzle::setup_puzzle_library, session::PuzzleSession};
use crate::visual::nodes::{GraphNode, NodeVisual, valence_to_color, update_node_visuals};
use crate::visual::physics::{NodePhysics, simulate_node_physics, apply_edge_spring_forces, apply_node_repulsion};
use crate::visual::interactions::{
    FleeMode, node_hover_flee, snap_back_from_flee, update_flee_target,
    DragState, HoverState, handle_pointer_input,
    trigger_trail_effects,
};
use crate::visual::edges::waves::{EdgeWaves, spawn_edge_waves, update_edge_waves};
use crate::visual::setup::{check_level_progression, setup_puzzle, setup_scene};
use crate::visual::sdf::sync::update_sdf_scene;
use crate::visual::ui::{spawn_hud, update_hud, HudTransitionState};
use bevy::prelude::*;

pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DragState>()
            .init_resource::<HoverState>()
            .init_resource::<EdgeWaves>()
            .init_resource::<FleeMode>()
            .init_resource::<HudTransitionState>()
            // Load puzzle library first, then set up initial puzzle and scene
            .add_systems(
                Startup,
                (setup_puzzle_library, setup_puzzle, setup_scene, spawn_hud).chain(),
            )
            .add_systems(
                Update,
                (
                    handle_pointer_input,
                    // Interaction effects
                    trigger_trail_effects,
                    spawn_edge_waves,
                    // Physics forces
                    apply_node_repulsion,
                    apply_edge_spring_forces,
                    simulate_node_physics,
                    update_flee_target, 
                    node_hover_flee,
                    snap_back_from_flee,
                    // Visual updates
                    update_node_visuals,
                    update_edge_waves,
                    update_sdf_scene,
                    snap_on_reset,
                    // HUD updates (unified seven-segment display)
                    update_hud,
                    // Level progression (check for completion and advance)
                    check_level_progression,
                )
                    .chain(),
            );
    }
}

/// Snap physics and colors back instantly when the board resets
fn snap_on_reset(
    session: Res<PuzzleSession>,
    mut nodes: Query<(&GraphNode, &mut NodePhysics, &mut NodeVisual)>,
) {
    // Only trigger when session has changed (reset happened)
    if !session.is_changed() {
        return;
    }

    // If trail is empty, a reset just happened - snap everything back
    if session.current_trail().is_empty() {
        for (graph_node, mut physics, mut visual) in &mut nodes {
            // Snap position back to rest instantly
            physics.position = physics.rest_position;
            physics.velocity = Vec3::ZERO;
            physics.forces = Vec3::ZERO;

            // Snap color back instantly
            let valence = session.current_valences().get(graph_node.node_id);
            visual.current_color = valence_to_color(valence);
        }
        info!("Snapped all nodes back to rest!");
    }
}

