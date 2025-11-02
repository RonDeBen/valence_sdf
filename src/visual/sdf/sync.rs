use bevy::prelude::*;

use crate::{
    game::session::PuzzleSession,
    visual::{
        nodes::{GraphNode, NodeVisual},
        interactions::pointer::{HoverState, DragState},
        physics::NodePhysics,
        edges::waves::EdgeWaves,
        sdf::material::{SceneMaterialHandle, SdfSceneMaterial},
        sdf::edges::cylinder::SdfCylinder,
    },
};

/// System: Update the unified SDF scene with all node and edge data
/// 
/// This syncs the ECS world state (physics, visuals, session) to the GPU shader uniforms.
pub fn update_sdf_scene(
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

