use crate::camera::{GameCamera, MainCamera};
use crate::input::{PointerEvent, PointerEventType};
use crate::sdf_material::{SdfSphereMaterial, SdfSphereMaterialData};
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;

pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DrawingState>()
            .add_systems(Startup, (setup_grid, setup_solution_display).chain())
            .add_systems(
                Update,
                (handle_pointer_input, debug_pointer, debug_draw_planes),
            );
    }
}

#[derive(Component)]
pub struct GraphNode {
    pub grid_position: (usize, usize),
    pub valence: i32,
}

#[derive(Resource, Default)]
struct DrawingState {
    is_drawing: bool,
    current_trail: Vec<Entity>,
}

/// Set up the 3x3 grid using camera bounds
fn setup_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut sdfs: ResMut<Assets<SdfSphereMaterial>>, // <-- use SDF assets
    game_camera: Res<GameCamera>,
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

    println!(
        "Grid setup: spacing={}, node_radius={}",
        spacing, node_radius
    );
    println!("Grid region: {:?}", grid_region);
    println!(
        "Grid will span from ({}, {}) to ({}, {})",
        start_x,
        start_z,
        start_x + (grid_size - 1) as f32 * spacing,
        start_z + (grid_size - 1) as f32 * spacing
    );

    let plane_size = spacing * 0.8;
    let plane_mesh = meshes.add(Plane3d::default().mesh().size(plane_size, plane_size));

    for row in 0..grid_size {
        for col in 0..grid_size {
            let center = Vec3::new(
                start_x + col as f32 * spacing,
                node_radius, // sits “on” the ground
                start_z + row as f32 * spacing,
            );

            let color = match (row, col) {
                (0, 0) => Vec4::new(1.0, 0.0, 0.0, 1.0),
                (0, 1) => Vec4::new(0.0, 1.0, 0.0, 1.0),
                (0, 2) => Vec4::new(0.0, 0.0, 1.0, 1.0),
                (1, 0) => Vec4::new(1.0, 1.0, 0.0, 1.0),
                (1, 1) => Vec4::new(1.0, 0.0, 1.0, 1.0),
                (1, 2) => Vec4::new(0.0, 1.0, 1.0, 1.0),
                (2, 0) => Vec4::new(1.0, 0.5, 0.0, 1.0),
                (2, 1) => Vec4::new(0.5, 0.0, 1.0, 1.0),
                (2, 2) => Vec4::new(1.0, 1.0, 1.0, 1.0),
                _ => Vec4::splat(0.5).with_w(1.0),
            };

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
                // plane on the ground directly under the sphere center
                Transform::from_translation(Vec3::new(center.x, 0.0, center.z)),
                GraphNode {
                    grid_position: (row, col),
                    valence: 5,
                },
            ));
        }
    }
}

/// Set up the solution display area (top 30% of screen)
/// This is where you'd put SDF objects showing previous solutions
fn setup_solution_display(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_camera: Res<GameCamera>,
) {
    // Position solution display in top 30% with padding
    let display_region = game_camera.bounds.region(
        (0.0, 1.0), // Full width
        (0.7, 1.0), // Top 30%
        0.05,       // 5% padding
    );

    println!("Solution display region: {:?}", display_region);

    // Example: spawn a few small spheres as placeholders for solution history
    // Later these would be your SDF morphing objects
    // TEMPORARILY DISABLED to debug SDF positioning
    /*
    for i in 0..3 {
        let pos = Vec3::new(
            display_region.left + display_region.width() * (i as f32 + 1.0) / 4.0,
            0.0,
            (display_region.top + display_region.bottom) * 0.5,
        );

        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.3))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.8, 0.2),
                metallic: 0.5,
                perceptual_roughness: 0.3,
                ..default()
            })),
            Transform::from_translation(pos),
        ));
    }
    */
}

/// Handle pointer input for drawing trails
fn handle_pointer_input(
    mut pointer_events: MessageReader<PointerEvent>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    nodes_query: Query<(Entity, &Transform), With<GraphNode>>,
    mut drawing_state: ResMut<DrawingState>,
    mut gizmos: Gizmos,
) {
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    for event in pointer_events.read() {
        // Convert to world position
        let Some(world_pos) = event.to_world_position(camera, camera_transform) else {
            continue;
        };

        match event.event_type {
            PointerEventType::Down => {
                // Check if we're clicking on a node
                for (entity, transform) in &nodes_query {
                    let distance = world_pos.distance(transform.translation);
                    if distance < 0.5 {
                        // Hit threshold
                        drawing_state.is_drawing = true;
                        drawing_state.current_trail = vec![entity];
                        println!("Started drawing from node: {:?}", entity);
                        break;
                    }
                }
            }

            PointerEventType::Move => {
                if drawing_state.is_drawing {
                    // Check if we moved over a new node
                    for (entity, transform) in &nodes_query {
                        if drawing_state.current_trail.contains(&entity) {
                            continue; // Already in trail
                        }

                        let distance = world_pos.distance(transform.translation);
                        if distance < 0.5 {
                            drawing_state.current_trail.push(entity);
                            println!("Added node to trail: {:?}", entity);
                            break;
                        }
                    }

                    // Draw current trail with gizmos
                    if drawing_state.current_trail.len() > 1 {
                        for window in drawing_state.current_trail.windows(2) {
                            if let (Ok((_, t1)), Ok((_, t2))) =
                                (nodes_query.get(window[0]), nodes_query.get(window[1]))
                            {
                                gizmos.line(t1.translation, t2.translation, Color::WHITE);
                            }
                        }

                        // Draw line from last node to cursor
                        if let Some(t) = drawing_state
                            .current_trail
                            .last()
                            .and_then(|&e| nodes_query.get(e).ok().map(|(_, t)| t))
                        {
                            gizmos.line(t.translation, world_pos, Color::srgba(1.0, 1.0, 1.0, 0.5));
                        }
                    }
                }
            }

            PointerEventType::Up => {
                if drawing_state.is_drawing {
                    println!(
                        "Finished drawing! Trail length: {}",
                        drawing_state.current_trail.len()
                    );
                    drawing_state.is_drawing = false;
                    drawing_state.current_trail.clear();
                }
            }
        }
    }
}

/// Debug system to visualize pointer events
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
            // Draw a circle at pointer position
            let color = match event.event_type {
                PointerEventType::Down => Color::srgb(0.0, 1.0, 0.0),
                PointerEventType::Move => Color::srgb(1.0, 1.0, 0.0),
                PointerEventType::Up => Color::srgb(1.0, 0.0, 0.0),
            };

            // Create circle in XZ plane (flat on ground, facing camera above)
            // Default circles are in XY plane, so we rotate -90° around X axis
            let rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
            let isometry = Isometry3d::new(world_pos, rotation);

            // Radius in world units - with view height of 8, use 0.3 for a visible cursor circle
            gizmos.circle(isometry, 0.3, color);
        }
    }
}

// Debug: draw crosses showing where the planes are positioned
fn debug_draw_planes(
    mut gizmos: Gizmos,
    query: Query<&Transform, (With<GraphNode>, With<Mesh3d>)>,
) {
    for transform in query.iter() {
        let pos = transform.translation;
        // Draw a small cross at each plane's position (at y=0)
        let size = 0.15;
        gizmos.line(
            Vec3::new(pos.x - size, pos.y, pos.z),
            Vec3::new(pos.x + size, pos.y, pos.z),
            Color::srgb(1.0, 0.5, 0.0), // Orange
        );
        gizmos.line(
            Vec3::new(pos.x, pos.y, pos.z - size),
            Vec3::new(pos.x, pos.y, pos.z + size),
            Color::srgb(1.0, 0.5, 0.0), // Orange
        );
    }
}
