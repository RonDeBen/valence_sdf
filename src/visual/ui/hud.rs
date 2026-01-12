//! HUD system using unified seven-segment display rendering.
//!
//! Spawns one large plane that renders all HUD elements (digits and slashes)
//! from an array of HudInstance structs in the shader.

use bevy::prelude::*;

use crate::{
    camera::{CameraBounds, GameCamera},
    game::{progression::ProgressionTracker, session::PuzzleSession},
    visual::sdf::seven_segment::{Digit, HudInstance, MAX_HUD_INSTANCES, SevenSegmentMaterial},
};

use super::{
    hud_builder::build_instances_for_group,
    number_group::{HudStyle, level_group, progress_group},
};

/// Resource to store the handle to the HUD material
#[derive(Resource)]
pub struct HudMaterialHandle(pub Handle<SevenSegmentMaterial>);

/// Resource to track HUD state for transition animations
#[derive(Resource)]
pub struct HudTransitionState {
    /// Previous instances (for per-digit change detection)
    pub prev_instances: Vec<HudInstance>,
    /// Transition duration in seconds
    pub transition_duration: f32,
}

impl Default for HudTransitionState {
    fn default() -> Self {
        Self {
            prev_instances: Vec::new(),
            transition_duration: 0.8, // 800ms transitions
        }
    }
}

/// Categorizes the type of transition occurring in the HUD
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransitionType {
    /// No changes detected
    None,
    /// Level advanced (only animate increasing digits, skip found counter reset)
    LevelAdvance,
    /// Progress changed within same level (animate all changed digits)
    ProgressChange,
}

/// Configuration for a transition animation
struct TransitionConfig {
    ty: TransitionType,
}

/// Spawn the unified HUD plane
pub fn spawn_hud(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SevenSegmentMaterial>>,
    game_camera: Res<GameCamera>,
) {
    info!("🎨 Spawning unified HUD display...");

    let bounds = &game_camera.bounds;
    let plane_size_x = bounds.width();
    let plane_size_y = bounds.height();

    let plane_mesh = meshes.add(Plane3d::default().mesh().size(plane_size_x, plane_size_y));

    let hud_material = SevenSegmentMaterial::default();
    let material_handle = materials.add(hud_material);

    commands.insert_resource(HudMaterialHandle(material_handle.clone()));

    let cx = bounds.width() * 0.5;
    let cy = bounds.height() * 0.5;

    commands.spawn((
        Mesh3d(plane_mesh),
        MeshMaterial3d(material_handle),
        Transform::from_xyz(cx, cy, 0.5)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        Name::new("HUD Plane"),
    ));

    info!("✨ Unified HUD plane spawned!");
}

/// Update the HUD material with current game state and animate transitions
pub fn update_hud(
    time: Res<Time>,
    tracker: Res<ProgressionTracker>,
    session: Res<PuzzleSession>,
    game_camera: Res<GameCamera>,
    hud_handle: Res<HudMaterialHandle>,
    mut transition_state: ResMut<HudTransitionState>,
    mut materials: ResMut<Assets<SevenSegmentMaterial>>,
) {
    let Some(material) = materials.get_mut(&hud_handle.0) else {
        return;
    };

    // 1. Build current instances from game state
    let current_instances = build_current_instances(&game_camera.bounds, &tracker, &session);

    // 2. Detect transition type (level advance vs normal progress)
    let progress = session.progress();
    let level_completed = tracker.is_changed() && progress.solutions_found == 0;
    let transition_type = if level_completed {
        TransitionType::LevelAdvance
    } else {
        TransitionType::ProgressChange
    };

    // 3. Apply transitions to instances (each digit computes its own flows in shader)
    let animated_instances = apply_transitions(
        current_instances,
        &transition_state.prev_instances,
        transition_type,
        &time,
        &transition_state,
    );

    // 4. Update material
    update_material(material, &animated_instances, time.elapsed_secs());

    // 5. Store for next frame
    transition_state.prev_instances = animated_instances;

    // Optional: Log on changes
    if tracker.is_changed() || session.is_changed() {
        let progress = session.progress();
        info!(
            "🔢 HUD updated: level={}, found={}/{}",
            tracker.current_level,
            progress.solutions_found,
            progress.total_solutions.unwrap_or(0)
        );
    }
}

/// Build HUD instances from current game state
fn build_current_instances(
    bounds: &CameraBounds,
    tracker: &ProgressionTracker,
    session: &PuzzleSession,
) -> Vec<HudInstance> {
    let style = HudStyle::default();
    let progress = session.progress();

    let groups = [
        level_group(tracker.current_level),
        progress_group(
            progress.solutions_found,
            progress.total_solutions.unwrap_or(0),
        ),
    ];

    let mut instances = Vec::new();
    for group in &groups {
        build_instances_for_group(bounds, group, style, &mut instances);
    }
    instances
}


/// Apply transition logic to instances based on transition type
fn apply_transitions(
    mut current: Vec<HudInstance>,
    previous: &[HudInstance],
    transition_type: TransitionType,
    time: &Time,
    state: &HudTransitionState,
) -> Vec<HudInstance> {
    match transition_type {
        TransitionType::None => current,
        TransitionType::LevelAdvance => {
            animate_increasing_digits(&mut current, previous, time, state);
            current
        }
        TransitionType::ProgressChange => {
            animate_all_changed(&mut current, previous, time, state);
            current
        }
    }
}

/// Animate only digits that increased in value (for level advance)
fn animate_increasing_digits(
    current: &mut [HudInstance],
    previous: &[HudInstance],
    time: &Time,
    state: &HudTransitionState,
) {
    for (inst, prev) in current.iter_mut().zip(previous.iter()) {
        if inst.kind != 0 {
            continue;
        }

        if prev.transition_progress < 1.0 {
            // Continue existing transition
            inst.from_mask = prev.from_mask;
            inst.transition_progress =
                (prev.transition_progress + time.delta_secs() / state.transition_duration).min(1.0);
        } else if inst.mask != prev.mask {
            let curr_val = Digit::from_mask(inst.mask as u8)
                .map(|d| d.to_u8())
                .unwrap_or(255);
            let prev_val = Digit::from_mask(prev.mask as u8)
                .map(|d| d.to_u8())
                .unwrap_or(255);

            if curr_val > prev_val {
                // Start new transition (increasing)
                inst.from_mask = prev.mask;
                inst.transition_progress = 0.0;
            } else {
                // Skip transition (decreasing)
                inst.from_mask = inst.mask;
                inst.transition_progress = 1.0;
            }
        }
    }
}

/// Animate all changed digits (for normal progress changes)
fn animate_all_changed(
    current: &mut [HudInstance],
    previous: &[HudInstance],
    time: &Time,
    state: &HudTransitionState,
) {
    for (inst, prev) in current.iter_mut().zip(previous.iter()) {
        if inst.kind != 0 {
            continue;
        }

        if prev.transition_progress < 1.0 {
            // Continue existing transition
            inst.from_mask = prev.from_mask;
            inst.transition_progress =
                (prev.transition_progress + time.delta_secs() / state.transition_duration).min(1.0);
        } else if inst.mask != prev.mask {
            // Start new transition
            inst.from_mask = prev.mask;
            inst.transition_progress = 0.0;
        }
    }
}

/// Update the material with animated instances
fn update_material(material: &mut SevenSegmentMaterial, instances: &[HudInstance], time: f32) {
    // Update instances
    let count = instances.len().min(MAX_HUD_INSTANCES);
    material.data.hud_count = count as u32;
    material.data.hud = [HudInstance::default(); MAX_HUD_INSTANCES];
    for (i, inst) in instances.iter().enumerate().take(MAX_HUD_INSTANCES) {
        material.data.hud[i] = *inst;
    }

    // Update time
    material.data.time = time;
}
