use crate::camera::{GameCamera, MainCamera};
use crate::game::session::{PuzzleSession, SessionResult};
use crate::graph::{NodeId, Valences};
use crate::input::{PointerEvent, PointerEventType};
use crate::sdf_material::{SceneMaterialHandle, SdfCylinder, SdfSceneMaterial, SdfSphere};
use crate::visual::node_physics::update_node_visuals;
use crate::visual::physics::debug::debug_physics;
use crate::visual::physics::edge_spring_forces::apply_edge_spring_forces;
use crate::visual::physics::node_interactions::trigger_node_interactions;
use crate::visual::physics::node_repulsion::apply_node_repulsion;
use crate::visual::physics::{NodePhysics, NodeVisual, simulate_node_physics};
use bevy::prelude::*;

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

pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DragState>()
            .init_resource::<HoverState>()
            .init_resource::<EdgeWaves>()
            .init_resource::<FleeMode>()
            .add_systems(Startup, (setup_puzzle, setup_scene).chain())
            .add_systems(
                Update,
                (
                    handle_pointer_input,
                    // draw_trail_preview,
                    debug_pointer,
                    // Physics systems
                    trigger_node_interactions,
                    spawn_edge_waves,
                    // apply_edge_spring_forces,
                    apply_node_repulsion,
                    apply_edge_spring_forces,
                    simulate_node_physics,
                    node_hover_flee,
                    snap_back_from_flee, // NEW: Snap back after flee
                    // Visual updates
                    update_node_visuals,
                    update_edge_waves, // NEW: Update traveling waves
                    update_sdf_scene,
                    snap_on_reset,
                    // Debug
                    // debug_physics,
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

/// Resource to track which node the mouse is hovering over
#[derive(Resource, Default)]
struct HoverState {
    hovered_node: Option<NodeId>,
    cursor_world_pos: Option<Vec3>,
}

/// Resource to track traveling tension waves on edges
#[derive(Resource, Default)]
struct EdgeWaves {
    waves: Vec<EdgeWave>,
}

/// A traveling tension wave on an edge
#[derive(Clone)]
struct EdgeWave {
    from: NodeId,
    to: NodeId,
    progress: f32,  // 0.0 = at 'from', 1.0 = at 'to'
    amplitude: f32, // Wave strength (0.0 to 1.0)
    direction: f32, // 0.0 = from→to, 1.0 = to→from
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
            scene_material.data.spheres[node_id.index()] = SdfSphere {
                center,
                radius: node_radius,
                base_color: color,
                target_color: color,
                infection_progress: 1.0, // Start fully infected (instant color)
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
                    base_color: color,       // Infection: start color
                    current_color: color,    // Infection: current color
                    target_color: color,     // Infection: target color
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
    hover_state: Res<HoverState>,
    drag_state: Res<DragState>,
    edge_waves: Res<EdgeWaves>,
    mut materials: ResMut<Assets<SdfSceneMaterial>>,
    scene_handle: Res<SceneMaterialHandle>,
) {
    let Some(material) = materials.get_mut(&scene_handle.0) else {
        return;
    };

    // Update all sphere positions and visuals
    for (graph_node, physics, visual) in &nodes {
        // let idx = graph_node.node_id.0 as usize;
        let sphere = &mut material.data.spheres[graph_node.node_id.index()];

        // Update position from physics
        sphere.center = physics.position;

        // Update infection animation data
        sphere.base_color = visual.base_color;
        sphere.target_color = visual.target_color;
        sphere.infection_progress = visual.infection_progress;

        // Update visual effects
        sphere.ripple_phase = visual.ripple_phase;
        sphere.ripple_amplitude = visual.ripple_amplitude;
        sphere.spike_amount = visual.glow; // Repurpose spike_amount for glow effect

        // Update stretch/squeeze (don't stack them!)
        let speed = physics.velocity.length();

        if speed > 0.08 {
            sphere.stretch_direction = physics.velocity.normalize();
            sphere.stretch_factor = 1.0 + (speed * 0.5).min(0.8);
        }
        // If squeezed (from valence) and NOT moving fast, apply squeeze
        else if visual.squeeze_factor > 0.01 {
            sphere.stretch_direction = Vec3::Y;
            sphere.stretch_factor = 1.0 - (visual.squeeze_factor * 0.5); // Half strength squeeze
        }
        // Default: no distortion
        else {
            sphere.stretch_direction = Vec3::Y;
            sphere.stretch_factor = 1.0;
        }
    }

    // Update edge cylinders
    let edges = session.edges();
    let mut cylinder_count = edges.len();

    for (i, edge) in edges.edges_in_order().iter().enumerate().take(16) {
        // Save room for preview
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

            // Find active wave for this edge
            let mut wave_phase = -1.0; // -1.0 = no wave
            let mut wave_amplitude = 0.0;

            for wave in &edge_waves.waves {
                if wave.from == edge.from && wave.to == edge.to {
                    // Calculate wave position (0.0 to 1.0 along edge)
                    wave_phase = if wave.direction < 0.5 {
                        wave.progress // from→to
                    } else {
                        1.0 - wave.progress // to→from
                    };
                    wave_amplitude = wave.amplitude;
                    break;
                }
            }

            material.data.cylinders[i] = SdfCylinder {
                start,
                _padding1: 0.0,
                end,
                radius: 0.08,                   // Thin connecting edges
                color: blended_color,           // Gradient blend of connected nodes
                node_a_idx: edge.from.0 as u32, // Track which nodes this connects
                node_b_idx: edge.to.0 as u32,
                wave_phase,     // Wave position
                wave_amplitude, // Wave strength
            };
        }
    }

    // Add preview cylinder from last node to cursor
    if drag_state.is_dragging {
        let trail = session.current_trail();
        if let Some(&last_node_id) = trail.last() {
            if let Some(cursor_pos) = hover_state.cursor_world_pos {
                // Find last node data
                if let Some((_, physics, visual)) = nodes
                    .iter()
                    .find(|(node, _, _)| node.node_id == last_node_id)
                {
                    let last_pos = physics.position;
                    let last_color = visual.current_color;

                    // Create preview cylinder (constant radius, no thick ends)
                    material.data.cylinders[cylinder_count.min(16)] = SdfCylinder {
                        start: last_pos,
                        _padding1: 0.0,
                        end: cursor_pos,
                        radius: 0.08, // Same as regular edges
                        color: last_color * Vec4::new(1.0, 1.0, 1.0, 0.5), // Semi-transparent
                        node_a_idx: last_node_id.0 as u32,
                        node_b_idx: last_node_id.0 as u32, // Same = preview (shader detects this)
                        wave_phase: -1.0,                  // No wave on preview
                        wave_amplitude: 0.0,
                    };
                    cylinder_count += 1;
                }
            }
        }
    }

    material.data.num_cylinders = cylinder_count.min(17) as u32;
}

/// Handle pointer input for drawing trails
fn handle_pointer_input(
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

/// Make invalid nodes flee ONLY when flee mode is active
fn node_hover_flee(
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

/// Snap nodes back to rest when flee mode ends
fn snap_back_from_flee(
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
            let color = valence_to_color(valence);
            visual.base_color = color;
            visual.target_color = color;
            visual.current_color = color;
            visual.infection_progress = 1.0; // Fully "infected" (instant color)
        }
        info!("Snapped all nodes back to rest!");
    }
}

/// Spawn tension waves on edges when a node is clicked
fn spawn_edge_waves(session: Res<PuzzleSession>, mut edge_waves: ResMut<EdgeWaves>) {
    // Only spawn waves when session changes (node was clicked)
    if !session.is_changed() {
        return;
    }

    let trail = session.current_trail();
    let Some(&clicked_node) = trail.last() else {
        return;
    };

    // Spawn waves on all edges connected to the clicked node
    let edges = session.edges();
    for edge in edges.edges_in_order() {
        if edge.from == clicked_node {
            // Wave travels from→to
            edge_waves.waves.push(EdgeWave {
                from: edge.from,
                to: edge.to,
                progress: 0.0,
                amplitude: 1.0,
                direction: 0.0, // from→to
            });
        } else if edge.to == clicked_node {
            // Wave travels to→from (backwards)
            edge_waves.waves.push(EdgeWave {
                from: edge.from,
                to: edge.to,
                progress: 0.0,
                amplitude: 1.0,
                direction: 1.0, // to→from
            });
        }
    }
}

/// Update traveling tension waves on edges
fn update_edge_waves(time: Res<Time>, mut edge_waves: ResMut<EdgeWaves>) {
    let dt = time.delta_secs();

    // Update all active waves
    edge_waves.waves.retain_mut(|wave| {
        wave.progress += dt * 2.0; // Speed of wave travel
        wave.amplitude *= 0.95_f32.powf(dt * 60.0); // Exponential decay

        // Keep wave if it's still active
        wave.progress < 1.0 && wave.amplitude > 0.01
    });
}
