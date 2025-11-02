use bevy::prelude::*;

use crate::{
    camera::MainCamera,
    game::session::{PuzzleSession, SessionResult},
    graph::NodeId,
    input::{PointerEvent, PointerEventType},
    visual::{
        nodes::GraphNode,
        physics::NodePhysics,
        interactions::flee::FleeMode,
    },
};

#[derive(Resource, Default)]
pub struct DragState {
    pub is_dragging: bool,
}

#[derive(Resource, Default)]
pub struct HoverState {
    pub hovered_node: Option<NodeId>,
    pub cursor_world_pos: Option<Vec3>,
}

/// System: Handle pointer input for drawing trails
pub fn handle_pointer_input(
    mut pointer_events: MessageReader<PointerEvent>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    nodes_query: Query<(&GraphNode, &NodePhysics)>,
    mut session: ResMut<PuzzleSession>,
    mut drag_state: ResMut<DragState>,
    mut hover_state: ResMut<HoverState>,
    mut flee_mode: ResMut<FleeMode>,
) {
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    for event in pointer_events.read() {
        let Some(world_pos) = event.to_world_position(camera, camera_transform) else {
            continue;
        };

        // Update hover state (which node is closest to cursor)
        hover_state.cursor_world_pos = Some(world_pos);
        hover_state.hovered_node = nodes_query
            .iter()
            .min_by(|(_, physics_a), (_, physics_b)| {
                let dist_a = world_pos.distance(physics_a.position);
                let dist_b = world_pos.distance(physics_b.position);
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .filter(|(_, physics)| world_pos.distance(physics.position) < 1.0) // Only hover if within range
            .map(|(node, _)| node.node_id);

        match event.event_type {
            PointerEventType::Down => {
                // Check if we're clicking on a node to start dragging
                for (graph_node, physics) in &nodes_query {
                    let distance = world_pos.distance(physics.position);
                    if distance < 0.5 {
                        match session.add_node(graph_node.node_id) {
                            SessionResult::FirstNode(node) => {
                                info!("Started trail at node {}", node.0);
                                drag_state.is_dragging = true;
                                flee_mode.deactivate();
                            }
                            SessionResult::EdgeAdded(edge) => {
                                info!("Added edge: {}-{}", edge.from.0, edge.to.0);
                                drag_state.is_dragging = true;
                                flee_mode.deactivate(); // Success - deactivate flee mode
                            }
                            SessionResult::Complete {
                                solution: _,
                                is_new,
                            } => {
                                if is_new {
                                    info!("🎉 NEW SOLUTION FOUND! 🎉");
                                } else {
                                    info!("Solution completed (already found)");
                                }
                                info!("Progress: {}", session.progress().display_string());

                                // Auto-reset for next attempt
                                session.reset();
                                info!("Board reset - try to find another solution!");
                                drag_state.is_dragging = false;
                                flee_mode.deactivate();
                            }
                            SessionResult::Invalid(err) => {
                                warn!("❌ Invalid move attempted: {} - ACTIVATING FLEE MODE", err);
                                flee_mode.activate(graph_node.node_id);
                            }
                        }
                        break;
                    }
                }
            }

            PointerEventType::Move => {
                // If we're dragging, check if we're hovering over a new node
                if drag_state.is_dragging {
                    let trail = session.current_trail();
                    let last_node = trail.last().copied();

                    for (graph_node, physics) in &nodes_query {
                        let distance = world_pos.distance(physics.position);

                        // Check if we're close to a node and it's not the last node we added
                        if distance < 0.5 && Some(graph_node.node_id) != last_node {
                            match session.add_node(graph_node.node_id) {
                                SessionResult::EdgeAdded(edge) => {
                                    info!("Added edge: {}-{}", edge.from.0, edge.to.0);
                                    flee_mode.deactivate(); // Success - deactivate flee mode
                                }
                                SessionResult::Complete {
                                    solution: _,
                                    is_new,
                                } => {
                                    if is_new {
                                        info!("🎉 NEW SOLUTION FOUND! 🎉");
                                    } else {
                                        info!("Solution completed (already found)");
                                    }
                                    info!("Progress: {}", session.progress().display_string());

                                    // Auto-reset for next attempt
                                    session.reset();
                                    info!("Board reset - try to find another solution!");
                                    drag_state.is_dragging = false;
                                    flee_mode.deactivate();
                                }
                                SessionResult::Invalid(err) => {
                                    // Activate flee mode on invalid attempt
                                    info!(
                                        "❌ Invalid move attempted: {} - ACTIVATING FLEE MODE",
                                        err
                                    );
                                    flee_mode.activate(graph_node.node_id);
                                }
                                _ => {}
                            }
                            break;
                        }
                    }
                }
            }

            PointerEventType::Up => {
                // Stop dragging and reset for next attempt
                drag_state.is_dragging = false;
                let trail_length = session.current_trail().len();

                // Deactivate flee mode when user releases
                if flee_mode.active {
                    info!("User released pointer - deactivating flee mode");
                    flee_mode.deactivate();
                }

                if trail_length > 0 {
                    session.reset();
                }
            }
        }
    }
}

