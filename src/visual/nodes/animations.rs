use bevy::prelude::*;

use crate::{
    game::session::PuzzleSession,
    visual::{
        nodes::{GraphNode, valence_to_color, components::NodeVisual},
        physics::NodePhysics,
        utils::{ease_in_out_cubic, lerp_hsv},
    },
};

/// System: Update visual animation states (color infection, squeeze, ripple decay)
pub fn update_node_visuals(
    time: Res<Time>,
    session: Res<PuzzleSession>,
    mut nodes: Query<(&GraphNode, &NodePhysics, &mut NodeVisual)>,
) {
    let dt = time.delta_secs();
    let valences = session.current_valences();

    for (graph_node, physics, mut visual) in &mut nodes {
        let valence = valences.get(graph_node.node_id);

        // === Color Infection Animation ===
        let new_target = valence_to_color(valence);

        if (new_target - visual.target_color).length() > 0.1 {
            visual.base_color = visual.current_color;
            visual.target_color = new_target;
            visual.infection_progress = 0.0;
        }

        if visual.infection_progress < 1.0 {
            visual.infection_progress += dt * 2.5;
            visual.infection_progress = visual.infection_progress.min(1.0);
        }

        // === EASING FUNCTION (try different ones!) ===
        // Options: ease_in_out_cubic (smooth S-curve), ease_out_cubic (fast start),
        //          ease_out_quad (gentler), linear (constant speed)
        let eased_progress = ease_in_out_cubic(visual.infection_progress);
        
        // Use HSV lerp for smoother color transitions around the color wheel!
        visual.current_color = lerp_hsv(visual.base_color, visual.target_color, eased_progress);

        // === Glow Decay (rapid fade) ===
        if visual.glow > 0.0 {
            // Fast exponential decay for snappy feedback
            visual.glow *= 0.95_f32.powf(dt * 60.0);  // Exponential decay
            
            // Clamp to zero when nearly invisible
            if visual.glow < 0.01 {
                visual.glow = 0.0;
            }
        }

        // === Squeeze from valence ===
        visual.target_squeeze = match valence {
            0 => 0.3,
            1 => 0.1,
            _ => 0.0,
        };
        visual.squeeze_factor = visual.squeeze_factor.lerp(visual.target_squeeze, dt * 2.0);

        // === Velocity squash ===
        let speed = physics.velocity.length();
        if speed > 0.2 && visual.target_squeeze < 0.05 {
            let velocity_squeeze = (speed * 0.05).min(0.3);
            visual.squeeze_factor = visual.squeeze_factor.max(velocity_squeeze);
        }

        if visual.ripple_amplitude > 0.01 {
            // Advance phase for gentle bounce effect (~3.5 seconds total)
            visual.ripple_phase += dt * 9.0; // Faster advance for shorter duration

            // Faster decay to complete in 3.5 seconds
            visual.ripple_amplitude *= 0.96; // Faster decay (was 0.98)

            // Debug: log ripple state occasionally
            if visual.ripple_phase < 0.1 {
                // Only log at the very start
                info!(
                    "🌊 Node {} rippling: phase={:.2}, amplitude={:.2}",
                    graph_node.node_id.0, visual.ripple_phase, visual.ripple_amplitude
                );
            }
        } else {
            visual.ripple_amplitude = 0.0;
        }
    }
}

