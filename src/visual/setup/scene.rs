use bevy::prelude::*;

use crate::{
    camera::GameCamera,
    game::session::PuzzleSession,
    graph::NodeId,
    visual::{
        nodes::{GraphNode, NodeVisual, valence_to_color},
        physics::NodePhysics,
        sdf::material::{DigitUvs, SceneMaterialHandle, SdfSceneMaterial},
        sdf::nodes::ellipsoid::SdfSphere,
        sdf::numbers::DigitAtlas,
    },
};

/// Node radius as a fraction of grid spacing
const NODE_RADIUS_FRACTION_OF_SPACING: f32 = 0.3;

/// How much larger the SDF plane is than the visible region
const PLANE_SIZE_SCALE: f32 = 1.5;

/// Extra spacing divisor so nodes don't touch the region edges
const SPACING_DENOMINATOR_OFFSET: f32 = 1.0;

/// Resource to store scene metrics for physics scaling
#[derive(Resource, Debug, Clone, Copy)]
pub struct SceneMetrics {
    /// Grid spacing (distance between nodes)
    pub spacing: f32,
}

impl SceneMetrics {
    pub fn new(spacing: f32) -> Self {
        Self { spacing }
    }
}

pub fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SdfSceneMaterial>>,
    asset_server: Res<AssetServer>,
    game_camera: Res<GameCamera>,
    session: Res<PuzzleSession>,
) {
    let grid_region = game_camera.bounds.region(0.0, 1.0, 0.0, 1.0, 0.0);

    let grid_size = 3;
    let grid_node_count = grid_size * grid_size;
    let node_id_row_stride = grid_size;

    let available_width = grid_region.width();
    let available_height = grid_region.height();

    let spacing =
        available_width.min(available_height) / (grid_size as f32 + SPACING_DENOMINATOR_OFFSET);
    let node_radius = spacing * NODE_RADIUS_FRACTION_OF_SPACING;

    let grid_width = (grid_size - 1) as f32 * spacing;
    let grid_height = (grid_size - 1) as f32 * spacing;

    // Center the grid both horizontally and vertically
    let start_x = grid_region.left + (grid_region.width() - grid_width) * 0.5;
    let start_y = grid_region.bottom + (grid_region.height() - grid_height) * 0.15;

    info!(
        "Scene setup: spacing={}, node_radius={}",
        spacing, node_radius
    );
    info!("Grid region: {:?}", grid_region);

    // Store scene metrics as a resource for physics scaling
    commands.insert_resource(SceneMetrics::new(spacing));

    let plane_size = grid_region.width().max(grid_region.height()) * PLANE_SIZE_SCALE;
    let plane_mesh = meshes.add(Plane3d::default().mesh().size(plane_size, plane_size));

    let digit_atlas = DigitAtlas::load(&asset_server);
    let digit_uvs = DigitUvs {
        uvs: digit_atlas.to_shader_uvs(),
    };

    let mut scene_material = SdfSceneMaterial::default();
    scene_material.data.num_spheres = grid_node_count as u32;
    scene_material.digit_atlas = digit_atlas.texture.clone();
    scene_material.digit_uvs = digit_uvs;

    commands.insert_resource(digit_atlas);

    let valences = session.current_valences();

    for row in 0..grid_size {
        for col in 0..grid_size {
            let node_id = NodeId(row * node_id_row_stride + col);
            let valence = valences.get(node_id);

            let center = Vec3::new(
                start_x + col as f32 * spacing,
                start_y + row as f32 * spacing,
                0.0, // Board is on XY plane at z=0
            );

            let color = valence_to_color(valence);

            scene_material.data.spheres[node_id.index()] = SdfSphere {
                center,
                radius: node_radius,
                color,
                stretch_direction: Vec3::Y,
                stretch_factor: 1.0,
                ripple_phase: 0.0,
                ripple_amplitude: 0.0,
                spike_amount: 0.0,
                digit_value: valence as u32,
            };

            // Scale spring stiffness by spacing for resolution-independent physics
            let mut physics = NodePhysics {
                position: center,
                rest_position: center,
                ..default()
            };
            physics.spring_stiffness *= spacing;

            commands.spawn((
                GraphNode { node_id },
                physics,
                NodeVisual {
                    current_color: color,
                    ..default()
                },
            ));

            info!(
                "Node {} at ({}, {}) - valence: {}",
                node_id.0, row, col, valence
            );
        }
    }

    let material_handle = materials.add(scene_material);
    commands.insert_resource(SceneMaterialHandle(material_handle.clone()));

    // Center the plane and rotate from XZ to XY
    let cx = grid_region.width() * 0.5;
    let cy = grid_region.height() * 0.5;

    commands.spawn((
        Mesh3d(plane_mesh),
        MeshMaterial3d(material_handle),
        Transform::from_xyz(cx, cy, 0.0)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
    ));

    info!("Unified SDF scene created!");
}
