use bevy::prelude::*;

use crate::{
    game::session::PuzzleSession,
    graph::NodeId,
};

/// Resource to track traveling tension waves on edges
#[derive(Resource, Default)]
pub struct EdgeWaves {
    pub(crate) waves: Vec<EdgeWave>,
}

/// A traveling tension wave on an edge
#[derive(Clone)]
pub(crate) struct EdgeWave {
    pub from: NodeId,
    pub to: NodeId,
    pub progress: f32,  // 0.0 = at 'from', 1.0 = at 'to'
    pub amplitude: f32, // Wave strength (0.0 to 1.0)
    pub direction: f32, // 0.0 = from→to, 1.0 = to→from
}

/// System: Spawn tension waves on edges when a node is clicked
pub fn spawn_edge_waves(session: Res<PuzzleSession>, mut edge_waves: ResMut<EdgeWaves>) {
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

/// System: Update traveling tension waves on edges
pub fn update_edge_waves(time: Res<Time>, mut edge_waves: ResMut<EdgeWaves>) {
    let dt = time.delta_secs();

    // Update all active waves
    edge_waves.waves.retain_mut(|wave| {
        wave.progress += dt * 2.0; // Speed of wave travel
        wave.amplitude *= 0.95_f32.powf(dt * 60.0); // Exponential decay

        // Keep wave if it's still active
        wave.progress < 1.0 && wave.amplitude > 0.01
    });
}

