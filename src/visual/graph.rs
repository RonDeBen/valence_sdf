use crate::camera::{GameCamera, MainCamera};
use crate::game::session::{PuzzleSession, SessionResult};
use crate::graph::{NodeId, Valences};
use crate::input::{PointerEvent, PointerEventType};
use crate::sdf_material::{SceneMaterialHandle, SdfCylinder, SdfSceneMaterial, SdfSphere};
use crate::visual::node_physics::{
    apply_edge_spring_forces, apply_node_repulsion, debug_physics, simulate_node_physics,
    trigger_node_interactions, update_node_visuals, NodePhysics, NodeVisual,
};
use bevy::prelude::*;

pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DragState>()
            .add_systems(Startup, (setup_puzzle, setup_scene).chain())
            .add_systems(
                Update,
                (
                    handle_pointer_input,
                    draw_trail_preview,
                    debug_pointer,
                    // Physics systems
                    trigger_node_interactions,
                    apply_edge_spring_forces,
                    apply_node_repulsion,
                    simulate_node_physics,
                    // Visual updates
                    update_node_visuals,
                    update_sdf_scene, // NEW: Update the unified scene
                    // Debug
                    debug_physics,
                )
                    .chain(), // Run in order
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

/// Setup the unified SDF scene with one large plane
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SdfSceneMaterial>>,
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
        "Scene setup: spacing={}, node_radius={}",
        spacing, node_radius
    );
    info!("Grid region: {:?}", grid_region);

    // Create ONE large plane that covers the whole game area
    let plane_size = grid_region.width().max(grid_region.height()) * 1.5;
    let plane_mesh = meshes.add(Plane3d::default().mesh().size(plane_size, plane_size));

    // Initialize the scene material with all spheres
    let mut scene_material = SdfSceneMaterial::default();
    scene_material.data.num_spheres = 9;

    let valences = session.current_valences();

    // Spawn all node entities (but don't attach individual meshes)
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

            // Initialize the sphere data in the material
            let idx = node_id.0 as usize;
            scene_material.data.spheres[idx] = SdfSphere {
                center,
                radius: node_radius,
                base_color: color,
                target_color: color,
                infection_progress: 1.0,  // Start fully infected (instant color)
                _padding1: 0.0,
                _padding2: 0.0,
                _padding3: 0.0,
                stretch_direction: Vec3::Y,
                stretch_factor: 1.0,
                ripple_phase: 0.0,
                ripple_amplitude: 0.0,
                spike_amount: 0.0,
                _padding: 0.0,
            };

            // Spawn node entity (no mesh, just data)
            commands.spawn((
                GraphNode { node_id },
                NodePhysics {
                    position: center,
                    rest_position: center,
                    ..default()
                },
                NodeVisual {
                    base_radius: node_radius,
                    base_color: color,      // Infection: start color
                    current_color: color,   // Infection: current color
                    target_color: color,    // Infection: target color
                    infection_progress: 1.0, // Start fully infected
                    ..default()
                },
            ));

            info!(
                "Node {} at ({}, {}) - valence: {}",
                node_id.0, row, col, valence
            );
        }
    }

    // Create the material and store its handle
    let material_handle = materials.add(scene_material);
    commands.insert_resource(SceneMaterialHandle(material_handle.clone()));

    // Spawn ONE plane with the unified scene material
    commands.spawn((
        Mesh3d(plane_mesh),
        MeshMaterial3d(material_handle),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    info!("Unified SDF scene created!");
}

/// Convert valence to color
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

/// Update the unified SDF scene with all node and edge data
fn update_sdf_scene(
    nodes: Query<(&GraphNode, &NodePhysics, &NodeVisual)>,
    session: Res<PuzzleSession>,
    mut materials: ResMut<Assets<SdfSceneMaterial>>,
    scene_handle: Res<SceneMaterialHandle>,
) {
    let Some(material) = materials.get_mut(&scene_handle.0) else {
        return;
    };

    // Update all sphere positions and visuals
    for (graph_node, physics, visual) in &nodes {
        let idx = graph_node.node_id.0 as usize;
        let sphere = &mut material.data.spheres[idx];

        // Update position from physics
        sphere.center = physics.position;

        // Update infection animation data
        sphere.base_color = visual.base_color;
        sphere.target_color = visual.target_color;
        sphere.infection_progress = visual.infection_progress;

        // Update visual effects
        sphere.ripple_phase = visual.ripple_phase;
        sphere.ripple_amplitude = visual.ripple_amplitude;
        sphere.spike_amount = if visual.is_invalid { 1.0 } else { 0.0 };

        // Update stretch/squeeze (don't stack them!)
        let speed = physics.velocity.length();
        
        // If moving fast, stretch in direction of motion
        if speed > 0.15 {
            sphere.stretch_direction = physics.velocity.normalize();
            sphere.stretch_factor = 1.0 + (speed * 0.15).min(0.3);  // Cap at 30% stretch
        } 
        // If squeezed (from valence) and NOT moving fast, apply squeeze
        else if visual.squeeze_factor > 0.01 {
            sphere.stretch_direction = Vec3::Y;
            sphere.stretch_factor = 1.0 - (visual.squeeze_factor * 0.5);  // Half strength squeeze
        } 
        // Default: no distortion
        else {
            sphere.stretch_direction = Vec3::Y;
            sphere.stretch_factor = 1.0;
        }
    }

    // Update edge cylinders
    let edges = session.edges();
    material.data.num_cylinders = edges.len().min(17) as u32;

    for (i, edge) in edges.edges_in_order().iter().enumerate().take(17) {
        // Find positions and colors of connected nodes
        let start_data = nodes
            .iter()
            .find(|(node, _, _)| node.node_id == edge.from)
            .map(|(_, physics, visual)| (physics.position, visual.current_color));

        let end_data = nodes
            .iter()
            .find(|(node, _, _)| node.node_id == edge.to)
            .map(|(_, physics, visual)| (physics.position, visual.current_color));

        if let (Some((start, start_color)), Some((end, end_color))) = (start_data, end_data) {
            // Blend the two node colors for a gradient effect
            let blended_color = (start_color + end_color) * 0.5;
            
            material.data.cylinders[i] = SdfCylinder {
                start,
                _padding1: 0.0,
                end,
                radius: 0.08, // Thin connecting edges
                color: blended_color, // Gradient blend of connected nodes
                node_a_idx: edge.from.0 as u32,  // Track which nodes this connects
                node_b_idx: edge.to.0 as u32,
                _padding2: 0,
                _padding3: 0,
            };
        }
    }
}

/// Handle pointer input for drawing trails
fn handle_pointer_input(
    mut pointer_events: MessageReader<PointerEvent>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    nodes_query: Query<(&GraphNode, &NodePhysics)>,
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
                for (graph_node, physics) in &nodes_query {
                    let distance = world_pos.distance(physics.position);
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

                    for (graph_node, physics) in &nodes_query {
                        let distance = world_pos.distance(physics.position);

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
    nodes_query: &Query<(&GraphNode, &NodePhysics)>,
) -> Option<Vec3> {
    nodes_query
        .iter()
        .find(|(node, _)| node.node_id == node_id)
        .map(|(_, physics)| physics.position)
}

/// Draw the trail preview
fn draw_trail_preview(
    session: Res<PuzzleSession>,
    drag_state: Res<DragState>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    nodes_query: Query<(&GraphNode, &NodePhysics)>,
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
