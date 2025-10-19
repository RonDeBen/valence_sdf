#import bevy_pbr::{
    mesh_view_bindings::view,
    forward_io::VertexOutput,
}

struct SdfSphereMaterialData {
    color: vec4<f32>,
    center: vec3<f32>,
    radius: f32,

    stretch_direction: vec3<f32>,
    stretch_factor: f32,

    ripple_phase: f32,
    ripple_amplitude: f32,
    spike_amount: f32,
    _padding: f32,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> material: SdfSphereMaterialData;

/// SDF for ellipsoid with squash/stretch (volume preserving)
fn sdf_ellipsoid(p: vec3<f32>, center: vec3<f32>, radius: f32,
                 stretch_dir: vec3<f32>, stretch: f32) -> f32 {
    let local_p = p - center;

    // Volume-preserving deformation
    // If we stretch by factor s in one direction, compress by sqrt(1/s) in others
    let compress = 1.0 / sqrt(stretch);

    // Decompose into parallel and perpendicular components
    let parallel = dot(local_p, stretch_dir) * stretch_dir;
    let perpendicular = local_p - parallel;

    // Apply anisotropic scaling
    let deformed = parallel * stretch + perpendicular * compress;

    return length(deformed) - radius;
}

/// Add ripple distortion
fn apply_ripple(base_sdf: f32, p: vec3<f32>, center: vec3<f32>) -> f32 {
    if (material.ripple_amplitude < 0.01) {
        return base_sdf;
    }

    let dist_from_center = length(p - center);
    let wave = sin(dist_from_center * 10.0 - material.ripple_phase * 5.0);
    let falloff = exp(-material.ripple_phase * 2.0);

    return base_sdf + wave * material.ripple_amplitude * falloff;
}

/// Add spiky distortion for invalid nodes
fn apply_spikes(base_sdf: f32, p: vec3<f32>, center: vec3<f32>) -> f32 {
    if (material.spike_amount < 0.01) {
        return base_sdf;
    }

    let local_p = p - center;
    // Simple noise-like spikes using sin functions
    let spike = sin(local_p.x * 20.0) * sin(local_p.y * 20.0) * sin(local_p.z * 20.0);

    return base_sdf - spike * material.spike_amount * 0.1;
}

fn sdf_sphere_world(p: vec3<f32>) -> f32 {
    // Base ellipsoid with squash/stretch
    var d = sdf_ellipsoid(
        p,
        material.center,
        material.radius,
        material.stretch_direction,
        material.stretch_factor
    );

    // Apply ripple
    d = apply_ripple(d, p, material.center);

    // Apply spikes if invalid
    d = apply_spikes(d, p, material.center);

    return d;
}

fn raymarch(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 64; i++) {
        let d = sdf_sphere_world(ro + rd * t);
        if (d < 0.001) { return t; }
        t += d;
        if (t > 200.0) { break; }
    }
    return -1.0;
}

fn normal_at(p: vec3<f32>) -> vec3<f32> {
    let e = 0.001;
    let dx = sdf_sphere_world(vec3(p.x + e, p.y, p.z))
           - sdf_sphere_world(vec3(p.x - e, p.y, p.z));
    let dy = sdf_sphere_world(vec3(p.x, p.y + e, p.z))
           - sdf_sphere_world(vec3(p.x, p.y - e, p.z));
    let dz = sdf_sphere_world(vec3(p.x, p.y, p.z + e))
           - sdf_sphere_world(vec3(p.x, p.y, p.z - e));
    return normalize(vec3(dx, dy, dz));
}

struct FragOut {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

@fragment
fn fragment(in: VertexOutput) -> FragOut {
    let cam = view.world_position;
    let ro = cam;
    let rd = normalize(in.world_position.xyz - cam);

    let t = raymarch(ro, rd);

    if (t > 0.0) {
        let hit = ro + rd * t;
        let n = normal_at(hit);
        let lambert = max(dot(n, normalize(vec3(1.0, 1.0, 1.0))), 0.0);
        let lighting = 0.3 + 0.7 * lambert;

        let clip = view.clip_from_world * vec4<f32>(hit, 1.0);
        let depth = clip.z / clip.w;

        return FragOut(vec4(material.color.rgb * lighting, material.color.a), depth);
    }

    discard;
}
