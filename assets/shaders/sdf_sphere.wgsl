#import bevy_pbr::{
    mesh_view_bindings::view,
    forward_io::VertexOutput,
}

struct SdfSphereMaterialData {
    color: vec4<f32>,
    center: vec3<f32>,
    radius: f32,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> material: SdfSphereMaterialData;

fn sdf_sphere_world(p: vec3<f32>, c: vec3<f32>, r: f32) -> f32 {
    return length(p - c) - r;
}

fn raymarch(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 64; i++) {
        let d = sdf_sphere_world(ro + rd * t, material.center, material.radius);
        if d < 0.001 { return t; }
        t += d;
        if t > 200.0 { break; }
    }
    return -1.0;
}

fn normal_at(p: vec3<f32>) -> vec3<f32> {
    let e = 0.001;
    let dx = sdf_sphere_world(vec3(p.x + e, p.y, p.z), material.center, material.radius)
           - sdf_sphere_world(vec3(p.x - e, p.y, p.z), material.center, material.radius);
    let dy = sdf_sphere_world(vec3(p.x, p.y + e, p.z), material.center, material.radius)
           - sdf_sphere_world(vec3(p.x, p.y - e, p.z), material.center, material.radius);
    let dz = sdf_sphere_world(vec3(p.x, p.y, p.z + e), material.center, material.radius)
           - sdf_sphere_world(vec3(p.x, p.y, p.z - e), material.center, material.radius);
    return normalize(vec3(dx, dy, dz));
}

// Output color + depth so multiple SDF planes overlap correctly
struct FragOut {
  @location(0) color: vec4<f32>,
  @builtin(frag_depth) depth: f32,
}

@fragment
fn fragment(in: VertexOutput) -> FragOut {
    // Camera world position from Bevy
    let cam = view.world_position;
    // Ray from camera to this fragment's world position (on your plane)
    let ro = cam;
    let rd = normalize(in.world_position.xyz - cam);
    // March
    let t = raymarch(ro, rd);
    // Hit → shade + write depth
    if t > 0.0 {
        let hit = ro + rd * t;
        // simple lighting
        let n = normal_at(hit);
        let lambert = max(dot(n, normalize(vec3(1.0, 1.0, 1.0))), 0.0);
        let lighting = 0.3 + 0.7 * lambert;
        // convert world → clip → depth (0..1)
        let clip = view.clip_from_world * vec4<f32>(hit, 1.0);
        let depth = clip.z / clip.w;
        return FragOut(vec4(material.color.rgb * lighting, material.color.a), depth);
    }
    // Miss → discard (transparent)
    discard;
}
