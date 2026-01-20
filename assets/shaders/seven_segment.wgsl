//! Seven-segment display shader with animated digit transitions.
//!
//! This shader renders 7-segment style digits with blob-style transition animations.
//! Features include: splitting, flying blobs, morphing, mass scaling, and breathing effects.

// ===== IMPORTS FROM BEVY =====
#import bevy_pbr::{
    mesh_view_bindings::view,
    forward_io::VertexOutput,
}

// ===== UNIFORM DATA =====
const MAX_FLOWS: u32 = 16u;

struct Flow {
    from_seg: u32,
    to_seg: u32,
    share: f32,
}

/// A single HUD element instance (digit or slash)
struct HudInstance {
    kind: u32,               // 0 = digit, 1 = slash
    mask: u32,               // Current/target mask
    from_mask: u32,          // Previous mask (for transitions)
    transition_progress: f32,// 0.0 = from_mask, 1.0 = mask
    pos: vec2<f32>,          // Position in world XY space
    scale: f32,
    _pad1: u32,
    _pad2: u32,
    _pad3: u32,
    _pad4: u32,
}

struct SevenSegmentData {
    time: f32,
    hud_count: u32,
    _padding1: u32,
    _padding2: u32,
    hud: array<HudInstance, 12>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> data: SevenSegmentData;

// ===== CONSTANTS =====
struct AnimationPhase {
    split_start: f32,       // 0.0
    split_end: f32,         // 0.15
    anticipation_end: f32,  // 0.25
    jump_end: f32,          // 0.70
    impact_end: f32,        // 0.85
    settle_end: f32,        // 1.0
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
    3.0     // breathing frequency
);

struct SettleConfig {
    enable_spiral: bool,
    spiral_rotations: f32, // How many full rotations (0 = no spiral, 2 = two spins)
    spiral_radius: f32,    // How far from center (0.0 = tight, 0.2 = wide)
    morph_curve: f32,      // How the morph accelerates (1.0 = linear, 2.0 = ease-in, 0.5 = ease-out)
}

const SETTLE: SettleConfig = SettleConfig(
    true,   // enable_spiral
    0.3,    // spiral_rotations (half a turn - try 0.0 to disable)
    0.04,   // spiral_radius (small - try 0.02 for tighter, 0.08 for wider)
    1.0     // morph_curve (slight ease-in - try 1.0 for linear)
);

// ===== SHADOW CONFIG =====
const SHADOW_OFFSET: vec2<f32> = vec2<f32>(0.09, -0.09);
const SHADOW_SOFTNESS: f32 = 0.05;
const SHADOW_OPACITY: f32 = 0.50;
const SHADOW_COLOR: vec3<f32> = vec3<f32>(0.1, 0.7, 0.7);

// ===== CREASE / BEVEL CONFIG =====
const RIM_WIDTH: f32 = 0.06;        // thickness in SDF units (try 0.03..0.10)
const CREASE_SHARPNESS: f32 = 2.2;   // higher => tighter highlight/shadow
const HIGHLIGHT_STRENGTH: f32 = 0.22;
const SHADOW_STRENGTH: f32 = 0.28;

// Optional: a subtle “core” darkening so segments feel rounded
const CORE_DARKEN_STRENGTH: f32 = 0.08;
const CORE_RADIUS: f32 = 0.18;       // how deep into the interior it affects

// ===== AXIS GROOVE CONFIG =====
// Width is relative to the segment radius (0.0..1.0ish)
const GROOVE_WIDTH: f32 = 0.85;     // 0.25 (thin) .. 0.6 (wide)
const GROOVE_DEPTH: f32 = 0.012;    // SDF units; 0.006..0.02 is typical
const GROOVE_SHARPNESS: f32 = 8.0;  // higher => tighter falloff


// ===== HELPERS =====
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

fn blend_crease_mask(d1: f32, d2: f32, k: f32) -> f32 {
    // d1 = smallest SDF, d2 = second smallest
    // When d2 ~= d1, we are near an overlap/crease.
    // Map |d2 - d1| in [0..k] -> [1..0]
    let gap = abs(d2 - d1);
    let m = clamp(1.0 - gap / max(k, 1e-5), 0.0, 1.0);

    // Optional: focus it near the surface, not deep inside
    // (crease should live around the boundary zone)
    let near_surface = 1.0 - smoothstep(0.10, 0.25, abs(d1)); // tweak
    return m * near_surface;
}


fn get_current_phase(t: f32) -> u32 {
    if t < PHASES.split_end { return 0u; }        // SPLIT
    if t < PHASES.anticipation_end { return 1u; } // ANTICIPATION
    if t < PHASES.jump_end { return 2u; }         // JUMP
    if t < PHASES.impact_end { return 3u; }       // IMPACT
    if t < PHASES.settle_end { return 4u; }       // SETTLE
    return 5u;                                    // DONE
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

// ===== SDF EFFECTS =====
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

    let offset = vec2<f32>(noise_x, noise_y) * 0.01 * intensity;
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
            seg.start = vec2<f32>(-seg_width + inline_gap, seg_length + corner_gap);
            seg.end = vec2<f32>(seg_width - inline_gap, seg_length + corner_gap);
        }
        case 1u: { // top-right
            seg.start = vec2<f32>(seg_width, seg_length - corner_gap);
            seg.end = vec2<f32>(seg_width, corner_gap);
        }
        case 2u: { // bottom-right
            seg.start = vec2<f32>(seg_width, -corner_gap);
            seg.end = vec2<f32>(seg_width, -seg_length + corner_gap);
        }
        case 3u: { // bottom
            seg.start = vec2<f32>(seg_width - inline_gap, -seg_length - corner_gap);
            seg.end = vec2<f32>(-seg_width + inline_gap, -seg_length - corner_gap);
        }
        case 4u: { // bottom-left
            seg.start = vec2<f32>(-seg_width, -seg_length + corner_gap);
            seg.end = vec2<f32>(-seg_width, -corner_gap);
        }
        case 5u: { // top-left
            seg.start = vec2<f32>(-seg_width, corner_gap);
            seg.end = vec2<f32>(-seg_width, seg_length - corner_gap);
        }
        case 6u: { // middle
            seg.start = vec2<f32>(-seg_width + inline_gap, 0.0);
            seg.end = vec2<f32>(seg_width - inline_gap, 0.0);
        }
        default: {
            seg.start = vec2<f32>(0.0, 0.0);
            seg.end = vec2<f32>(0.0, 0.0);
        }
    }
    return seg;
}

fn hash(n: u32) -> f32 {
    let x = ((n * 1103515245u + 12345u) & 0x7fffffffu);
    return f32(x) / f32(0x7fffffffu);
}

// Get a pseudo-random value for a specific flow
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

    // Random-ish offset along segment length
    let along = 0.5 + pseudo_gaussian(flow_idx, segment_id) * 0.2;
    let t = clamp(along, 0.2, 0.8);  // Keep it away from very ends

    return mix(seg.start, seg.end, t);
}

fn seg_mid(segment_id: u32) -> vec2<f32> {
    let seg = get_segment_geometry(segment_id);
    return (seg.start + seg.end) * 0.5;
}

// ===== FLOW COMPUTATION (PER-DIGIT) =====
fn get_active_segments(mask: u32) -> array<u32, 7> {
    var is_active: array<u32, 7>;
    for (var i = 0u; i < 7u; i++) {
        is_active[i] = select(0u, 1u, (mask & (1u << i)) != 0u);
    }
    return is_active;
}

fn segment_distance(from_seg: u32, to_seg: u32) -> u32 {
    if from_seg == to_seg { return 0u; }

    // 0=Top, 1=TopRight, 2=BottomRight, 3=Bottom, 4=BottomLeft, 5=TopLeft, 6=Middle
    let adjacent = array<array<u32, 3>, 7>(
        array<u32, 3>(1u, 5u, 6u), // Top
        array<u32, 3>(0u, 2u, 6u), // TopRight
        array<u32, 3>(1u, 3u, 6u), // BottomRight
        array<u32, 3>(2u, 4u, 6u), // Bottom
        array<u32, 3>(3u, 5u, 6u), // BottomLeft
        array<u32, 3>(0u, 4u, 6u), // TopLeft
        array<u32, 3>(0u, 1u, 2u)  // Middle (simplified)
    );

    for (var i = 0u; i < 3u; i++) {
        if adjacent[from_seg][i] == to_seg {
            return 1u;
        }
    }
    return 2u;
}

fn compute_flows(from_mask: u32, to_mask: u32) -> array<Flow, MAX_FLOWS> {
    var flows: array<Flow, MAX_FLOWS>;
    var flow_count = 0u;

    let from_active = get_active_segments(from_mask);
    let to_active = get_active_segments(to_mask);

    // Disappearing segments: route to nearest target segments
    for (var from_seg = 0u; from_seg < 7u; from_seg++) {
        if from_active[from_seg] == 1u && to_active[from_seg] == 0u {
            var min_dist = 999u;
            var nearest_count = 0u;

            // Pass 1: find min distance
            for (var to_seg = 0u; to_seg < 7u; to_seg++) {
                if to_active[to_seg] == 1u {
                    let dist = segment_distance(from_seg, to_seg);
                    min_dist = min(min_dist, dist);
                }
            }

            // Pass 2: count how many at min distance
            for (var to_seg = 0u; to_seg < 7u; to_seg++) {
                if to_active[to_seg] == 1u {
                    let dist = segment_distance(from_seg, to_seg);
                    if dist == min_dist {
                        nearest_count++;
                    }
                }
            }

            // Pass 3: create flows with equal share
            let share = 1.0 / f32(max(nearest_count, 1u));

            for (var to_seg = 0u; to_seg < 7u; to_seg++) {
                if to_active[to_seg] == 1u && flow_count < MAX_FLOWS {
                    let dist = segment_distance(from_seg, to_seg);
                    if dist == min_dist {
                        flows[flow_count] = Flow(from_seg, to_seg, share);
                        flow_count++;
                    }
                }
            }
        }
    }

    // “Excitement flows”: stable -> appearing
    for (var to_seg = 0u; to_seg < 7u; to_seg++) {
        if from_active[to_seg] == 0u && to_active[to_seg] == 1u && flow_count < MAX_FLOWS {
            var min_dist = 999u;
            var closest_stable = 0u;

            for (var stable_seg = 0u; stable_seg < 7u; stable_seg++) {
                if from_active[stable_seg] == 1u && to_active[stable_seg] == 1u {
                    let dist = segment_distance(stable_seg, to_seg);
                    if dist < min_dist {
                        min_dist = dist;
                        closest_stable = stable_seg;
                    }
                }
            }

            if min_dist < 999u {
                flows[flow_count] = Flow(closest_stable, to_seg, 0.2);
                flow_count++;
            }
        }
    }

    return flows;
}

fn count_flows(flows: array<Flow, MAX_FLOWS>) -> u32 {
    var count = 0u;
    for (var i = 0u; i < MAX_FLOWS; i++) {
        if flows[i].share > 0.0 {
            count++;
        }
    }
    return count;
}

// ===== RENDER A SINGLE SEGMENT =====
// fn segment_sdf(p: vec2<f32>, segment_id: u32, radius: f32) -> f32 {
//     let seg = get_segment_geometry(segment_id);
//     return sd_capsule(p, seg.start, seg.end, radius);
// }

fn segment_sdf(p: vec2<f32>, segment_id: u32, radius: f32) -> f32 {
    let seg = get_segment_geometry(segment_id);

    // Capsule distance (same math as sd_capsule)
    let pa = p - seg.start;
    let ba = seg.end - seg.start;

    let h = clamp(dot(pa, ba) / max(dot(ba, ba), 1e-5), 0.0, 1.0);
    let closest = pa - ba * h;

    // Distance from point to the capsule axis (centerline)
    let axis_dist = length(closest);

    // Base capsule SDF
    var d = axis_dist - radius;

    // ---- Axis groove: shallow indentation along the centerline ----
    // Normalized radius (0 at centerline, ~1 at edge)
    let r = axis_dist / max(radius, 1e-5);

    // A peaked profile at the centerline
    // (1 - r) keeps it inside, pow() controls how “pointy” it is.
    let groove_profile = pow(clamp(1.0 - r / max(GROOVE_WIDTH, 1e-5), 0.0, 1.0), GROOVE_SHARPNESS);

    // Add a small positive bump => SDF is less negative along the axis
    // => looks like a groove under lighting because gradients bend around it.
    d += GROOVE_DEPTH * groove_profile;

    return d;
}


fn render_segment_with_effects(
    p: vec2<f32>,
    segment_id: u32,
    base_radius: f32,
    mass: f32,
    distortion_intensity: f32
) -> f32 {
    // 1. Apply position distortion FIRST (modifies input space)
    let distorted_p = apply_distortion(p, data.time, distortion_intensity);

    // 2. Radius with mass + breathing
    var radius = base_radius;
    radius = apply_mass_scaling(radius, mass);
    radius = apply_breathing(radius, segment_id, distorted_p, data.time);

    // 3. SDF
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
                1.0,  // full mass
                0.0   // no distortion static
            );
            d = smin(d, seg_sdf, 0.18);
        }
    }

    return d;
}

fn render_transition(p: vec2<f32>, from_mask: u32, to_mask: u32, t: f32) -> f32 {
    if t <= 0.0 { return render_static_digit(p, from_mask); }
    if t >= 1.0 { return render_static_digit(p, to_mask); }

    let flows = compute_flows(from_mask, to_mask);
    let flow_count = count_flows(flows);

    let phase = get_current_phase(t);
    let phase_progress = get_phase_progress(t, phase);

    switch phase {
        case 0u: { return render_split_phase(p, from_mask, to_mask, phase_progress); }
        case 1u: { return render_anticipation_phase(p, from_mask, to_mask, phase_progress); }
        case 2u: { return render_jump_phase(p, from_mask, to_mask, phase_progress, flows, flow_count); }
        case 3u: { return render_impact_phase(p, from_mask, to_mask, phase_progress, flows, flow_count); }
        case 4u: { return render_settle_phase(p, from_mask, to_mask, phase_progress, flows, flow_count); }
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
                // stays on
                let seg_sdf = render_segment_with_effects(p, seg_id, base_radius, 1.0, 0.0);
                d = smin(d, seg_sdf, 0.18);
            } else {
                // shrinking
                let shrink_factor = 1.0 - (phase_progress * 0.7);
                if shrink_factor > 0.01 {
                    let seg_sdf = render_segment_with_effects(p, seg_id, base_radius, shrink_factor, 0.0);
                    d = smin(d, seg_sdf, 0.18);
                }
            }
        }
    }

    return d;
}

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
                // stays on
                let seg_sdf = render_segment_with_effects(p, seg_id, base_radius, 1.0, 0.0);
                d = smin(d, seg_sdf, 0.18);
            } else {
                // shrink + wobble
                let shrink = 0.3 - (phase_progress * 0.2);

                let wobble = vec2<f32>(
                    sin(data.time * 5.0 + f32(seg_id)),
                    cos(data.time * 4.0 + f32(seg_id))
                ) * 0.02 * phase_progress;

                let wobbled_p = p + wobble;

                let seg_sdf = render_segment_with_effects(wobbled_p, seg_id, base_radius, shrink, 0.0);
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
    phase_progress: f32,
    flows: array<Flow, MAX_FLOWS>,
    flow_count: u32
) -> f32 {
    var d = 1e9;
    let base_radius = 0.12;

    // stable segments
    for (var seg_id = 0u; seg_id < 7u; seg_id++) {
        let from_active = (from_mask & (1u << seg_id)) != 0u;
        let to_active = (to_mask & (1u << seg_id)) != 0u;

        if from_active && to_active {
            let seg_sdf = render_segment_with_effects(p, seg_id, base_radius, 1.0, 0.0);
            d = smin(d, seg_sdf, 0.18);
        }
    }

    // traveling blobs
    for (var i = 0u; i < flow_count; i++) {
        let flow = flows[i];

        let from_pos = seg_target_point(flow.from_seg, i);
        let to_pos = seg_target_point(flow.to_seg, i + 100u);

        let start_offset = hash_flow(i, 1u) * 0.15;
        let adjusted_progress = clamp(
            (phase_progress - start_offset) / (1.0 - start_offset),
            0.0,
            1.0
        );

        let eased = ease_varied(adjusted_progress, i);
        let blob_pos = mix(from_pos, to_pos, eased);

        let distortion_strength = sin(adjusted_progress * 3.14159) * 0.3;
        let distorted_p = apply_distortion(p, data.time, distortion_strength);

        var blob_radius = base_radius * 1.2;
        blob_radius = apply_mass_scaling(blob_radius, flow.share);

        let blob_sdf = length(distorted_p - blob_pos) - blob_radius;
        d = smin(d, blob_sdf, 0.18);
    }

    return d;
}

fn render_impact_phase(
    p: vec2<f32>,
    from_mask: u32,
    to_mask: u32,
    phase_progress: f32,
    flows: array<Flow, MAX_FLOWS>,
    flow_count: u32
) -> f32 {
    var d = 1e9;
    let base_radius = 0.12;

    // stable segments only
    for (var seg_id = 0u; seg_id < 7u; seg_id++) {
        let from_active = (from_mask & (1u << seg_id)) != 0u;
        let to_active = (to_mask & (1u << seg_id)) != 0u;

        if from_active && to_active {
            let seg_sdf = render_segment_with_effects(p, seg_id, base_radius, 1.0, 0.0);
            d = smin(d, seg_sdf, 0.18);
        }
    }

    // squashing blobs at destination
    for (var i = 0u; i < flow_count; i++) {
        let flow = flows[i];
        let to_pos = seg_target_point(flow.to_seg, i + 100u);

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
    phase_progress: f32,
    flows: array<Flow, MAX_FLOWS>,
    flow_count: u32
) -> f32 {
    var d = 1e9;
    let base_radius = 0.12;

    // stable segments
    for (var seg_id = 0u; seg_id < 7u; seg_id++) {
        let from_active = (from_mask & (1u << seg_id)) != 0u;
        let to_active = (to_mask & (1u << seg_id)) != 0u;

        if from_active && to_active {
            let seg_sdf = render_segment_with_effects(p, seg_id, base_radius, 1.0, 0.0);
            d = smin(d, seg_sdf, 0.18);
        }
    }

    // morph blob -> segment
    for (var i = 0u; i < flow_count; i++) {
        let flow = flows[i];

        let target_seg = get_segment_geometry(flow.to_seg);
        let blob_pos = seg_target_point(flow.to_seg, i + 100u);

        let eased_progress = phase_progress * phase_progress; // ease-in quadratic

        let start_pos = mix(blob_pos, target_seg.start, eased_progress);
        let end_pos = mix(blob_pos, target_seg.end, eased_progress);

        let morphing_sdf = sd_capsule(p, start_pos, end_pos, base_radius);
        d = smin(d, morphing_sdf, 0.18);
    }

    return d;
}

// ===== SLASH RENDERING =====
fn sd_capsule_2d(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h) - r;
}

fn render_slash(p: vec2<f32>) -> f32 {
    let a = vec2<f32>(-0.4, -0.6);
    let b = vec2<f32>(0.4, 0.6);
    let r = 0.12;
    return sd_capsule_2d(p, a, b, r);
}

fn scene_sdf(p_world: vec2<f32>) -> f32 {
    var min_d = 1e9;

    for (var i = 0u; i < data.hud_count; i++) {
        let inst = data.hud[i];
        let local_p = (p_world - inst.pos) / max(inst.scale, 0.001);

        var d: f32;
        if inst.kind == 1u {
            d = render_slash(local_p) * inst.scale;
        } else {
            d = render_transition(
                local_p / 1.2,
                inst.from_mask,
                inst.mask,
                inst.transition_progress
            ) * inst.scale;
        }

        min_d = min(min_d, d);
    }

    return min_d;
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
    let p = vec2<f32>(world_pos.x, world_pos.y);

    // Compute min distances for FG + shadow (single pass over instances)
    var min_d = 1e9;
    var min_shadow_d = 1e9;

    for (var i = 0u; i < data.hud_count; i++) {
        let inst = data.hud[i];
        let local_p = (p - inst.pos) / max(inst.scale, 0.001);

        // FG
        var d: f32;
        if inst.kind == 1u {
            d = render_slash(local_p) * inst.scale;
        } else {
            d = render_transition(local_p / 1.2, inst.from_mask, inst.mask, inst.transition_progress) * inst.scale;
        }
        min_d = min(min_d, d);

        // Shadow: sample shifted local coords
        let shadow_local_p = local_p - SHADOW_OFFSET;

        var sd: f32;
        if inst.kind == 1u {
            sd = render_slash(shadow_local_p) * inst.scale;
        } else {
            sd = render_transition(shadow_local_p / 1.2, inst.from_mask, inst.mask, inst.transition_progress) * inst.scale;
        }
        min_shadow_d = min(min_shadow_d, sd);
    }

    // Shading should be computed ONCE for the whole scene, not per-instance
    let d0 = scene_sdf(p);

    let eps = 0.003;
    let dx = scene_sdf(p + vec2<f32>(eps, 0.0)) - scene_sdf(p - vec2<f32>(eps, 0.0));
    let dy = scene_sdf(p + vec2<f32>(0.0, eps)) - scene_sdf(p - vec2<f32>(0.0, eps));
    let n = normalize(vec2<f32>(dx, dy) + vec2<f32>(1e-6, 1e-6));

    let light_dir = normalize(vec2<f32>(-1.0, 1.0));
    let ndl = clamp(dot(n, light_dir), 0.0, 1.0);

    let inside = smoothstep(0.0, 0.06, -d0);
    let lit = 0.75 + 0.45 * ndl;

    // Crease
    let rim = smoothstep(RIM_WIDTH, 0.0, -d0) * inside;

    let h = pow(ndl, CREASE_SHARPNESS) * rim;
    let s = pow(1.0 - ndl, CREASE_SHARPNESS) * rim;

    let core = smoothstep(0.0, CORE_RADIUS, -d0) * inside; // 0 near edge, 1 deeper inside


    // Alpha edges
    let fg_a = smoothstep(0.02, 0.0, min_d);
    let shadow_a_raw = smoothstep(SHADOW_SOFTNESS, 0.0, min_shadow_d);
    let shadow_a = shadow_a_raw * SHADOW_OPACITY * (1.0 - fg_a);

    // Composite
    var out_rgb = vec3<f32>(0.05, 0.05, 0.1);
    var out_a = 0.0;

    out_rgb = mix(out_rgb, SHADOW_COLOR, shadow_a);
    out_a = max(out_a, shadow_a);

    var fg_rgb = vec3<f32>(1.0) * mix(1.0, lit, inside);

    // Apply bevel: brighten on highlight rim, darken on shadow rim
    fg_rgb += vec3<f32>(1.0) * (HIGHLIGHT_STRENGTH * h);
    fg_rgb -= vec3<f32>(1.0) * (SHADOW_STRENGTH * s);

    out_rgb = mix(out_rgb, fg_rgb, fg_a);
    out_a = max(out_a, fg_a);

    if out_a > 0.01 {
        let clip = view.clip_from_world * vec4<f32>(world_pos, 1.0);
        let depth = clip.z / clip.w;

        return FragOut(vec4<f32>(out_rgb, out_a), depth);
    }

    return FragOut(vec4<f32>(0.0), 0.9999);
}
