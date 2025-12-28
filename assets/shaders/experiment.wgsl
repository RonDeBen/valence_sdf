// ===== IMPORTS FROM BEVY =====
#import bevy_pbr::{
    mesh_view_bindings::view,
    forward_io::VertexOutput,
}

// ===== UNIFORM DATA =====
struct ExperimentData {
    time: f32,
    from_digit: u32,
    to_digit: u32,
    transition_progress: f32,
    from_digit_mask: u32,
    to_digit_mask: u32,
    flow_count: u32,
    _padding: u32,
}

@group(2) @binding(0)
var<uniform> data: ExperimentData;

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

// ===== SEGMENT GEOMETRY =====
struct Segment {
    start: vec2<f32>,
    end: vec2<f32>,
}

fn get_segment_geometry(segment_id: u32) -> Segment {
    let seg_length = 0.5;
    let seg_width = 0.35;
    let corner_gap = 0.08;
    let inline_gap = 0.13;

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

// ===== RENDER A SINGLE SEGMENT =====
fn segment_sdf(p: vec2<f32>, segment_id: u32, radius: f32) -> f32 {
    let seg = get_segment_geometry(segment_id);
    return sd_capsule(p, seg.start, seg.end, radius);
}

// ===== RENDER A COMPLETE DIGIT =====
fn render_static_digit(p: vec2<f32>, mask: u32) -> f32 {
    var d = 1e9;  // Start with "very far away"

    let radius = 0.12;

    // For each of the 7 segments...
    for (var i = 0u; i < 7u; i++) {
        // Check if this segment is active in the digit
        let is_active = (mask & (1u << i)) != 0u;

        if is_active {
            // Get the SDF for this segment
            let seg_sdf = segment_sdf(p, i, radius);

            // Blend it with what we have so far
            d = smin(d, seg_sdf, 0.18);
        }
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

    // TEST: Render digit "8" (all segments on)
    let d = render_static_digit(p_scaled, 0x7Fu);  // 0b1111111

    let edge = smoothstep(0.02, 0.0, d);

    if edge > 0.01 {
        let clip = view.clip_from_world * vec4<f32>(world_pos, 1.0);
        let depth = clip.z / clip.w;
        return FragOut(vec4(1.0, 0.0, 0.0, edge), depth);
    }

    return FragOut(vec4(0.05, 0.05, 0.1, 0.0), 0.9999);
}
