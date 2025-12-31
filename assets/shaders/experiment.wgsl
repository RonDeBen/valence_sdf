// ===== IMPORTS FROM BEVY =====
#import bevy_pbr::{
    mesh_view_bindings::view,
    forward_io::VertexOutput,
}

// ===== UNIFORM DATA =====
const MAX_FLOWS: u32 = 16u;

struct ShaderFlow {
    from_seg: u32,
    to_seg: u32,
    share: f32,
    _padding: u32,
}

struct ExperimentData {
    time: f32,
    from_digit: u32,
    to_digit: u32,
    transition_progress: f32,
    from_digit_mask: u32,
    to_digit_mask: u32,
    flow_count: u32,
    _padding: u32,
    flows: array<ShaderFlow, MAX_FLOWS>,  // Fixed-size array
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> data: ExperimentData;


// constants

struct AnimationPhase {
    split_start: f32,     // 0.0
    split_end: f32,       // 0.15
    anticipation_end: f32, // 0.25
    jump_end: f32,        // 0.70
    impact_end: f32,      // 0.85
    settle_end: f32,      // 1.0
}

const PHASES: AnimationPhase = AnimationPhase(
    0.0,   // split starts
    0.15,  // split ends, anticipation starts
    0.25,  // anticipation ends, jump starts
    0.70,  // jump ends, impact starts
    0.80,  // impact ends, settle starts
    1.0    // settle ends
);

struct EffectConfig {
    enable_mass_scaling: bool,
    enable_breathing: bool,
    enable_distortion: bool,
    breathing_amplitude: f32,
    breathing_frequency: f32,
}

const EFFECTS: EffectConfig = EffectConfig(
    true,   // mass scaling
    true,   // breathing
    false,  // distortion (start disabled)
    0.15,   // breathing amplitude
    3.0,    // breathing frequency
);

struct SettleConfig {
    enable_spiral: bool,
    spiral_rotations: f32,    // How many full rotations (0 = no spiral, 2 = two spins)
    spiral_radius: f32,       // How far from center (0.0 = tight, 0.2 = wide)
    morph_curve: f32,         // How the morph accelerates (1.0 = linear, 2.0 = ease-in, 0.5 = ease-out)
}

const SETTLE: SettleConfig = SettleConfig(
    true,   // enable_spiral
    0.3,    // spiral_rotations (half a turn - try 0.0 to disable)
    0.04,   // spiral_radius (small - try 0.02 for tighter, 0.08 for wider)
    1.0,    // morph_curve (slight ease-in - try 1.0 for linear)
);

// helpers

fn ease_out_cubic(t: f32) -> f32 {
    let u = 1.0 - t;
    return 1.0 - u * u * u;
}

fn ease_varied(t: f32, blob_id: u32) -> f32 {
    // Mix between different easing functions based on blob ID
    let choice = hash_flow(blob_id, 10u);

    if choice < 0.33 {
        // Ease out cubic (slow end)
        let u = 1.0 - t;
        return 1.0 - u * u * u;
    } else if choice < 0.66 {
        // Ease in-out (slow start and end)
        if t < 0.5 {
            return 2.0 * t * t;
        } else {
            let u = 1.0 - t;
            return 1.0 - 2.0 * u * u;
        }
    } else {
        // Ease in cubic (slow start)
        return t * t * t;
    }
}

fn get_current_phase(t: f32) -> u32 {
    if t < PHASES.split_end { return 0u; }        // SPLIT
    if t < PHASES.anticipation_end { return 1u; } // ANTICIPATION
    if t < PHASES.jump_end { return 2u; }         // JUMP
    if t < PHASES.impact_end { return 3u; }       // IMPACT
    if t < PHASES.settle_end { return 4u; }       // IMPACT
    return 5u;                                    // SETTLE
}

fn get_phase_progress(t: f32, phase: u32) -> f32 {
    // Returns 0.0-1.0 for how far through THIS phase we are
    switch phase {
        case 0u: { // SPLIT
            return t / PHASES.split_end;
        }
        case 1u: { // ANTICIPATION
            return (t - PHASES.split_end) / (PHASES.anticipation_end - PHASES.split_end);
        }
        case 2u: { // JUMP
            return (t - PHASES.anticipation_end) / (PHASES.jump_end - PHASES.anticipation_end);
        }
        case 3u: { // IMPACT
            return (t - PHASES.jump_end) / (PHASES.impact_end - PHASES.jump_end);
        }
        case 4u: { // SETTLE
            return (t - PHASES.impact_end) / (PHASES.settle_end - PHASES.impact_end);
        }
        default: { return 0.0; }
    }
}

// ===== BASIC SDF FUNCTIONS =====
fn sd_capsule(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / max(dot(ba, ba), 1e-5), 0.0, 1.0);
    return length(pa - ba * h) - r;
}

fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h = max(k - abs(a - b), 0.0) / max(k, 1e-5);
    return min(a, b) - h * h * h * k * (1.0 / 6.0);
}

// SDF effects

fn apply_mass_scaling(base_radius: f32, mass: f32) -> f32 {
    if !EFFECTS.enable_mass_scaling {
        return base_radius;
    }

    // Mass affects radius via square root (area preservation)
    return base_radius * sqrt(mass);
}

fn apply_breathing(base_radius: f32, segment_id: u32, p: vec2<f32>, time: f32) -> f32 {
    if !EFFECTS.enable_breathing {
        return base_radius;
    }

    let seg = get_segment_geometry(segment_id);
    let center = (seg.start + seg.end) * 0.5;
    let axis = seg.end - seg.start;
    let length_seg = max(length(axis), 1e-5);
    let dir = axis / length_seg;

    // Position along segment (-1 to 1)
    let local = p - center;
    let along = dot(local, dir) / (0.5 * length_seg);

    // Wave pattern along the segment
    let phase = time * EFFECTS.breathing_frequency + f32(segment_id) * 0.7;
    let wave = sin(along * 3.0 + phase) * 0.5 + 0.5;  // 0 to 1

    let breath_mult = 1.0 + (wave - 0.5) * EFFECTS.breathing_amplitude;
    return base_radius * breath_mult;
}

fn apply_distortion(p: vec2<f32>, time: f32, intensity: f32) -> vec2<f32> {
    if !EFFECTS.enable_distortion || intensity < 0.01 {
        return p;
    }

    // Simple noise-based distortion
    let noise_x = sin(p.x * 8.0 + time) * cos(p.y * 6.0 - time * 0.7);
    let noise_y = cos(p.x * 6.0 - time * 0.8) * sin(p.y * 8.0 + time * 0.5);

    let offset = vec2(noise_x, noise_y) * 0.01 * intensity;
    return p + offset;
}

// ===== SEGMENT GEOMETRY =====
struct Segment {
    start: vec2<f32>,
    end: vec2<f32>,
}

fn get_segment_geometry(segment_id: u32) -> Segment {
    let seg_length = 0.5;
    let seg_width = 0.35;
    let corner_gap = 0.08;
    let inline_gap = 0.18;

    var seg: Segment;
    switch segment_id {
        case 0u: { // top
            seg.start = vec2(-seg_width + inline_gap, seg_length + corner_gap);
            seg.end = vec2(seg_width - inline_gap, seg_length + corner_gap);
        }
        case 1u: { // top-right
            seg.start = vec2(seg_width, seg_length - corner_gap);
            seg.end = vec2(seg_width, corner_gap);
        }
        case 2u: { // bottom-right
            seg.start = vec2(seg_width, -corner_gap);
            seg.end = vec2(seg_width, -seg_length + corner_gap);
        }
        case 3u: { // bottom
            seg.start = vec2(seg_width - inline_gap, -seg_length - corner_gap);
            seg.end = vec2(-seg_width + inline_gap, -seg_length - corner_gap);
        }
        case 4u: { // bottom-left
            seg.start = vec2(-seg_width, -seg_length + corner_gap);
            seg.end = vec2(-seg_width, -corner_gap);
        }
        case 5u: { // top-left
            seg.start = vec2(-seg_width, corner_gap);
            seg.end = vec2(-seg_width, seg_length - corner_gap);
        }
        case 6u: { // middle
            seg.start = vec2(-seg_width + inline_gap, 0.0);
            seg.end = vec2(seg_width - inline_gap, 0.0);
        }
        default: {
            seg.start = vec2(0.0, 0.0);
            seg.end = vec2(0.0, 0.0);
        }
    }
    return seg;
}

fn hash(n: u32) -> f32 {
    let x = ((n * 1103515245u + 12345u) & 0x7fffffffu);
    return f32(x) / f32(0x7fffffffu);
}

// Get a pseudo-random value for a specific flow
// Uses flow indices to get consistent randomness per flow
fn hash_flow(flow_idx: u32, seed: u32) -> f32 {
    return hash(flow_idx * 73u + seed * 37u);
}

// Approximate gaussian from uniform (Box-Muller lite)
fn pseudo_gaussian(flow_idx: u32, seed: u32) -> f32 {
    let u1 = hash_flow(flow_idx, seed);
    let u2 = hash_flow(flow_idx, seed + 1u);
    // Central limit theorem approximation: sum of uniforms → gaussian-ish
    return (u1 + u2 - 1.0) * 0.7;  // Range: roughly -0.7 to +0.7
}

fn seg_target_point(segment_id: u32, flow_idx: u32) -> vec2<f32> {
    let seg = get_segment_geometry(segment_id);

    // Get a random offset along the segment length
    // 0.0 = start, 0.5 = middle, 1.0 = end
    let along = 0.5 + pseudo_gaussian(flow_idx, segment_id) * 0.2;
    let t = clamp(along, 0.2, 0.8);  // Keep it away from very ends

    return mix(seg.start, seg.end, t);
}

fn seg_mid(segment_id: u32) -> vec2<f32> {
    let seg = get_segment_geometry(segment_id);
    return (seg.start + seg.end) * 0.5;
}

// ===== RENDER A SINGLE SEGMENT =====
fn segment_sdf(p: vec2<f32>, segment_id: u32, radius: f32) -> f32 {
    let seg = get_segment_geometry(segment_id);
    return sd_capsule(p, seg.start, seg.end, radius);
}

fn render_segment_with_effects(
    p: vec2<f32>,
    segment_id: u32,
    base_radius: f32,
    mass: f32,  // 1.0 for normal, can vary
    distortion_intensity: f32  // 0.0-1.0
) -> f32 {
    // 1. Apply position distortion FIRST (modifies input space)
    let distorted_p = apply_distortion(p, data.time, distortion_intensity);

    // 2. Calculate radius with mass and breathing
    var radius = base_radius;
    radius = apply_mass_scaling(radius, mass);
    radius = apply_breathing(radius, segment_id, distorted_p, data.time);

    // 3. Compute SDF with modified parameters
    return segment_sdf(distorted_p, segment_id, radius);
}

// ===== RENDERING =====
fn render_static_digit(p: vec2<f32>, mask: u32) -> f32 {
    var d = 1e9;
    let base_radius = 0.12;

    for (var i = 0u; i < 7u; i++) {
        let is_active = (mask & (1u << i)) != 0u;

        if is_active {
            let seg_sdf = render_segment_with_effects(
                p,
                i,
                base_radius,
                1.0,  // Full mass
                0.0   // No distortion when static
            );
            d = smin(d, seg_sdf, 0.18);
        }
    }

    return d;
}

fn render_transition(p: vec2<f32>, from_mask: u32, to_mask: u32, t: f32) -> f32 {
    if t <= 0.0 {
        return render_static_digit(p, from_mask);
    }
    if t >= 1.0 {
        return render_static_digit(p, to_mask);
    }

    let phase = get_current_phase(t);
    let phase_progress = get_phase_progress(t, phase);

    switch phase {
        case 0u: { return render_split_phase(p, from_mask, to_mask, phase_progress); }
        case 1u: { return render_anticipation_phase(p, from_mask, to_mask, phase_progress); }
        case 2u: { return render_jump_phase(p, from_mask, to_mask, phase_progress); }
        case 3u: { return render_impact_phase(p, from_mask, to_mask, phase_progress); }
        case 4u: { return render_settle_phase(p, from_mask, to_mask, phase_progress); }
        default: { return render_static_digit(p, to_mask); }
    }
}

fn render_split_phase(
    p: vec2<f32>,
    from_mask: u32,
    to_mask: u32,
    phase_progress: f32
) -> f32 {
    var d = 1e9;
    let base_radius = 0.12;

    for (var seg_id = 0u; seg_id < 7u; seg_id++) {
        let from_active = (from_mask & (1u << seg_id)) != 0u;
        let to_active = (to_mask & (1u << seg_id)) != 0u;

        if from_active {
            if to_active {
                // Stays ON - full breathing effect
                let seg_sdf = render_segment_with_effects(
                    p, seg_id, base_radius, 1.0, 0.0
                );
                d = smin(d, seg_sdf, 0.18);
            } else {
                // Shrinking - mass decreases, still breathing
                let shrink_factor = 1.0 - (phase_progress * 0.7);

                if shrink_factor > 0.01 {
                    let seg_sdf = render_segment_with_effects(
                        p, seg_id, base_radius, shrink_factor, 0.0
                    );
                    d = smin(d, seg_sdf, 0.18);
                }
            }
        }
    }

    return d;
}

// PHASE 1: ANTICIPATION (continue shrinking, add wobble)
fn render_anticipation_phase(
    p: vec2<f32>,
    from_mask: u32,
    to_mask: u32,
    phase_progress: f32
) -> f32 {
    var d = 1e9;
    let base_radius = 0.12;

    for (var seg_id = 0u; seg_id < 7u; seg_id++) {
        let from_active = (from_mask & (1u << seg_id)) != 0u;
        let to_active = (to_mask & (1u << seg_id)) != 0u;

        if from_active {
            if to_active {
                // Stays ON - breathing continues
                let seg_sdf = render_segment_with_effects(
                    p, seg_id, base_radius, 1.0, 0.0
                );
                d = smin(d, seg_sdf, 0.18);
            } else {
                // Continue shrinking: 0.3 → 0.1 + wobble position
                let shrink = 0.3 - (phase_progress * 0.2);

                // Add tiny wobble to the position
                let wobble = vec2(
                    sin(data.time * 5.0 + f32(seg_id)),
                    cos(data.time * 4.0 + f32(seg_id))
                ) * 0.02 * phase_progress;

                let wobbled_p = p + wobble;

                // Still has breathing even while wobbling
                let seg_sdf = render_segment_with_effects(
                    wobbled_p, seg_id, base_radius, shrink, 0.0
                );
                d = smin(d, seg_sdf, 0.18);
            }
        }
    }

    return d;
}

fn render_jump_phase(
    p: vec2<f32>,
    from_mask: u32,
    to_mask: u32,
    phase_progress: f32
) -> f32 {
    var d = 1e9;
    let base_radius = 0.12;

    // 1. Stable segments - keep breathing
    for (var seg_id = 0u; seg_id < 7u; seg_id++) {
        let from_active = (from_mask & (1u << seg_id)) != 0u;
        let to_active = (to_mask & (1u << seg_id)) != 0u;

        if from_active && to_active {
            let seg_sdf = render_segment_with_effects(
                p, seg_id, base_radius, 1.0, 0.0
            );
            d = smin(d, seg_sdf, 0.18);
        }
    }

    // 3. Traveling blobs with mass + optional distortion
    for (var i = 0u; i < data.flow_count; i++) {
        let flow = data.flows[i];

        let from_pos = seg_target_point(flow.from_seg, i);
        let to_pos = seg_target_point(flow.to_seg, i + 100u);

        let start_offset = hash_flow(i, 1u) * 0.15;
        let adjusted_progress = clamp((phase_progress - start_offset) / (1.0 - start_offset), 0.0, 1.0);
        let eased = ease_varied(adjusted_progress, i);

        let blob_pos = mix(from_pos, to_pos, eased);

        // Optional: slight distortion during flight
        let distortion_strength = sin(adjusted_progress * 3.14159) * 0.3;
        let distorted_p = apply_distortion(p, data.time, distortion_strength);

        // Apply mass scaling to blob radius
        var blob_radius = base_radius * 0.7;
        blob_radius = apply_mass_scaling(blob_radius, flow.share);

        let blob_sdf = length(distorted_p - blob_pos) - blob_radius;
        d = smin(d, blob_sdf, 0.18);
    }

    return d;
}

// PHASE 3: IMPACT (blobs arrive and squash)
fn render_impact_phase(
    p: vec2<f32>,
    from_mask: u32,
    to_mask: u32,
    phase_progress: f32
) -> f32 {
    var d = 1e9;
    let base_radius = 0.12;

    // 1. ONLY segments that stayed ON (with effects!)
    for (var seg_id = 0u; seg_id < 7u; seg_id++) {
        let from_active = (from_mask & (1u << seg_id)) != 0u;
        let to_active = (to_mask & (1u << seg_id)) != 0u;

        if from_active && to_active {
            // Apply effects to stable segments
            let seg_sdf = render_segment_with_effects(
                p, seg_id, base_radius, 1.0, 0.0
            );
            d = smin(d, seg_sdf, 0.18);
        }
        // DON'T render appeared segments here - blobs will become them!
    }

    // 2. Squashing blobs (these are the "appeared segments" in blob form)
    for (var i = 0u; i < data.flow_count; i++) {
        let flow = data.flows[i];
        let to_pos = seg_target_point(flow.to_seg, i + 100u);

        // Squash animation: 0.3 → 0.0
        let squash_amount = (1.0 - phase_progress) * 0.3;

        var radius = base_radius * (1.0 + squash_amount);
        radius = apply_mass_scaling(radius, flow.share);

        let blob_sdf = length(p - to_pos) - radius;
        d = smin(d, blob_sdf, 0.18);
    }

    return d;
}

fn render_settle_phase(
    p: vec2<f32>,
    from_mask: u32,
    to_mask: u32,
    phase_progress: f32
) -> f32 {
    var d = 1e9;
    let base_radius = 0.12;

    // 1. Stable segments - unchanged
    for (var seg_id = 0u; seg_id < 7u; seg_id++) {
        let from_active = (from_mask & (1u << seg_id)) != 0u;
        let to_active = (to_mask & (1u << seg_id)) != 0u;

        if from_active && to_active {
            let seg_sdf = render_segment_with_effects(
                p, seg_id, base_radius, 1.0, 0.0
            );
            d = smin(d, seg_sdf, 0.18);
        }
    }

    // 2. Simple expansion: blob point → full segment
    for (var i = 0u; i < data.flow_count; i++) {
        let flow = data.flows[i];
        let target_seg = get_segment_geometry(flow.to_seg);
        let blob_pos = seg_target_point(flow.to_seg, i + 100u);

        // Ease the expansion (optional: try linear first with phase_progress)
        let eased_progress = phase_progress * phase_progress;  // Ease-in quadratic

        // Start at blob_pos, expand to full segment
        let start_pos = mix(blob_pos, target_seg.start, eased_progress);
        let end_pos = mix(blob_pos, target_seg.end, eased_progress);

        // Simple capsule, no fancy effects
        let morphing_sdf = sd_capsule(p, start_pos, end_pos, base_radius);
        d = smin(d, morphing_sdf, 0.18);
    }

    return d;
}


// ===== FRAGMENT SHADER OUTPUT =====
struct FragOut {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

// ===== MAIN FRAGMENT SHADER =====
@fragment
fn fragment(in: VertexOutput) -> FragOut {
    let world_pos = in.world_position.xyz;
    let p = vec2(-world_pos.x, world_pos.z);
    let p_scaled = p / 1.2;

    // USE DATA FROM RUST (instead of hardcoded 0x7F)
    let d = render_transition(
        p_scaled,
        data.from_digit_mask,   // ← from Rust
        data.to_digit_mask,     // ← from Rust
        data.transition_progress // ← from Rust
    );

    let edge = smoothstep(0.02, 0.0, d);

    if edge > 0.01 {
        let clip = view.clip_from_world * vec4<f32>(world_pos, 1.0);
        let depth = clip.z / clip.w;
        return FragOut(vec4(1.0, 0.0, 0.0, edge), depth);
    }

    return FragOut(vec4(0.05, 0.05, 0.1, 0.0), 0.9999);
}
