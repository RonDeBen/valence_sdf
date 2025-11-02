use bevy::prelude::*;

use crate::{
    camera::GameCamera,
    game::session::PuzzleSession,
    graph::NodeId,
    visual::{
        nodes::{GraphNode, NodeVisual, valence_to_color},
        physics::NodePhysics,
        sdf::material::{SceneMaterialHandle, SdfSceneMaterial},
        sdf::nodes::ellipsoid::SdfSphere,
    },
};

/// System: Setup the unified SDF scene with one large plane
pub fn setup_scene(
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

