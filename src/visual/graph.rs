use crate::camera::{GameCamera, MainCamera};
use crate::game::session::{PuzzleSession, SessionResult};
use crate::graph::{NodeId, Valences};
use crate::input::{PointerEvent, PointerEventType};
use crate::sdf_material::{SdfSphereMaterial, SdfSphereMaterialData};
use bevy::prelude::*;

pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DragState>()
            .add_systems(Startup, (setup_puzzle, setup_grid).chain())
            .add_systems(
                Update,
                (
                    handle_pointer_input,
                    update_node_colors,
                    draw_trail_preview,
                    debug_pointer,
                ),
            );
    }
}

/// Component marking a visual node
#[derive(Component)]
pub struct GraphNode {
    pub node_id: NodeId,
}

/// Resource to track if we're currently dragging
#[derive(Resource, Default)]
struct DragState {
    is_dragging: bool,
}

/// Setup the puzzle session
fn setup_puzzle(mut commands: Commands) {
    // hardcoded puzzle for now
    let valences = Valences::new(vec![2, 4, 2, 4, 8, 4, 2, 5, 3]);
    let session = PuzzleSession::new(valences, 1);

    commands.insert_resource(session);

    info!("Puzzle loaded: 17-edge complexity");
}

/// Setup the visual 3x3 grid
fn setup_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut sdfs: ResMut<Assets<SdfSphereMaterial>>,
    game_camera: Res<GameCamera>,
    session: Res<PuzzleSession>,
) {
    let grid_region = game_camera.bounds.region((0.0, 1.0), (0.0, 0.6), 0.1);

    let grid_size = 3;
    let available_width = grid_region.width();
    let available_height = grid_region.height();
    let spacing = available_width.min(available_height) / (grid_size as f32 + 1.0);
    let node_radius = spacing * 0.3;

    let grid_width = (grid_size - 1) as f32 * spacing;
    let grid_height = (grid_size - 1) as f32 * spacing;
    let start_x = grid_region.left + (grid_region.width() - grid_width) * 0.5;
    let start_z = grid_region.bottom + (grid_region.height() - grid_height) * 0.5;

    info!(
        "Grid setup: spacing={}, node_radius={}",
        spacing, node_radius
    );
    info!("Grid region: {:?}", grid_region);

    let plane_size = spacing * 0.8;
    let plane_mesh = meshes.add(Plane3d::default().mesh().size(plane_size, plane_size));

    let valences = session.current_valences();

    for row in 0..grid_size {
        for col in 0..grid_size {
            let node_id = NodeId(row * 3 + col);
            let valence = valences.get(node_id);

            let center = Vec3::new(
                start_x + col as f32 * spacing,
                node_radius,
                start_z + row as f32 * spacing,
            );

            let color = valence_to_color(valence);

            let mat = sdfs.add(SdfSphereMaterial {
                data: SdfSphereMaterialData {
                    color,
                    center,
                    radius: node_radius,
                },
            });

            commands.spawn((
                Mesh3d(plane_mesh.clone()),
                MeshMaterial3d(mat),
                Transform::from_translation(Vec3::new(center.x, 0.0, center.z)),
                GraphNode { node_id },
            ));

            info!(
                "Node {} at ({}, {}) - valence: {}",
                node_id.0, row, col, valence
            );
        }
    }
}

/// Convert valence to color
fn valence_to_color(valence: usize) -> Vec4 {
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

/// Update node colors based on current valences
fn update_node_colors(
    session: Res<PuzzleSession>,
    mut nodes: Query<(&GraphNode, &mut MeshMaterial3d<SdfSphereMaterial>)>,
    mut materials: ResMut<Assets<SdfSphereMaterial>>,
) {
    // Only update if session changed
    if !session.is_changed() {
        return;
    }

    let valences = session.current_valences();

    for (node, material_handle) in &mut nodes {
        if let Some(material) = materials.get_mut(material_handle.id()) {
            let valence = valences.get(node.node_id);
            material.data.color = valence_to_color(valence);
        }
    }
}

/// Handle pointer input for drawing trails
fn handle_pointer_input(
    mut pointer_events: MessageReader<PointerEvent>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    nodes_query: Query<(&GraphNode, &Transform)>,
    mut session: ResMut<PuzzleSession>,
    mut drag_state: ResMut<DragState>,
) {
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    for event in pointer_events.read() {
        let Some(world_pos) = event.to_world_position(camera, camera_transform) else {
            continue;
        };

        match event.event_type {
            PointerEventType::Down => {
                // Check if we're clicking on a node to start dragging
                for (graph_node, transform) in &nodes_query {
                    let distance = world_pos.distance(transform.translation);
                    if distance < 0.5 {
                        match session.add_node(graph_node.node_id) {
                            SessionResult::FirstNode(node) => {
                                info!("Started trail at node {}", node.0);
                                drag_state.is_dragging = true;
                            }
                            SessionResult::EdgeAdded(edge) => {
                                info!("Added edge: {}-{}", edge.from.0, edge.to.0);
                                drag_state.is_dragging = true;
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
                            }
                            SessionResult::Invalid(err) => {
                                warn!("Invalid move: {}", err);
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

                    for (graph_node, transform) in &nodes_query {
                        let distance = world_pos.distance(transform.translation);

                        // Check if we're close to a node and it's not the last node we added
                        if distance < 0.5 && Some(graph_node.node_id) != last_node {
                            match session.add_node(graph_node.node_id) {
                                SessionResult::EdgeAdded(edge) => {
                                    info!("Added edge: {}-{}", edge.from.0, edge.to.0);
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
                                }
                                SessionResult::Invalid(_err) => {
                                    // Silently ignore invalid moves during drag (reduces spam)
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

                if trail_length > 0 {
                    info!("Released - resetting board (had {} nodes)", trail_length);
                    session.reset();
                }
            }
        }
    }
}

/// Helper to get node position by ID
fn get_node_position(
    node_id: NodeId,
    nodes_query: &Query<(&GraphNode, &Transform)>,
) -> Option<Vec3> {
    nodes_query
        .iter()
        .find(|(node, _)| node.node_id == node_id)
        .map(|(_, transform)| transform.translation)
}

/// Draw the trail preview
fn draw_trail_preview(
    session: Res<PuzzleSession>,
    drag_state: Res<DragState>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    nodes_query: Query<(&GraphNode, &Transform)>,
    mut pointer_events: MessageReader<PointerEvent>,
    mut gizmos: Gizmos,
) {
    let trail = session.current_trail();

    // Draw edges between nodes in the trail
    for window in trail.windows(2) {
        let Some(pos_a) = get_node_position(window[0], &nodes_query) else {
            continue;
        };
        let Some(pos_b) = get_node_position(window[1], &nodes_query) else {
            continue;
        };

        gizmos.line(pos_a, pos_b, Color::srgb(1.0, 1.0, 1.0));
    }

    // Draw preview line from last node to cursor while dragging
    if !drag_state.is_dragging {
        return;
    }

    let Some(&last_node) = trail.last() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    for event in pointer_events.read() {
        let Some(world_pos) = event.to_world_position(camera, camera_transform) else {
            continue;
        };

        let Some(last_pos) = get_node_position(last_node, &nodes_query) else {
            continue;
        };

        gizmos.line(last_pos, world_pos, Color::srgba(1.0, 1.0, 1.0, 0.4));
    }
}

// Keep your existing debug functions
fn debug_pointer(
    mut pointer_events: MessageReader<PointerEvent>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut gizmos: Gizmos,
) {
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    for event in pointer_events.read() {
        if let Some(world_pos) = event.to_world_position(camera, camera_transform) {
            let color = match event.event_type {
                PointerEventType::Down => Color::srgb(0.0, 1.0, 0.0),
                PointerEventType::Move => Color::srgb(1.0, 1.0, 0.0),
                PointerEventType::Up => Color::srgb(1.0, 0.0, 0.0),
            };

            let rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
            let isometry = Isometry3d::new(world_pos, rotation);
            gizmos.circle(isometry, 0.3, color);
        }
    }
}
