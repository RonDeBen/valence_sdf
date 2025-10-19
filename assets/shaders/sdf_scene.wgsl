#import bevy_pbr::{
    mesh_view_bindings::view,
    forward_io::VertexOutput,
}

struct SdfSphere {
    center: vec3<f32>,
    radius: f32,

    // Infection animation
    base_color: vec4<f32>,
    target_color: vec4<f32>,
    infection_progress: f32,
    _padding1: f32,
    _padding2: f32,
    _padding3: f32,

    stretch_direction: vec3<f32>,
    stretch_factor: f32,
    ripple_phase: f32,
    ripple_amplitude: f32,
    spike_amount: f32,
    _padding: f32,
}

struct SdfCylinder {
    start: vec3<f32>,
    _padding1: f32,
    end: vec3<f32>,
    radius: f32,
    color: vec4<f32>,

    // Track which nodes this connects
    node_a_idx: u32,
    node_b_idx: u32,
    _padding2: u32,
    _padding3: u32,
}

struct SdfSceneUniform {
    num_spheres: u32,
    num_cylinders: u32,
    _padding1: u32,
    _padding2: u32,
    spheres: array<SdfSphere, 9>,
    cylinders: array<SdfCylinder, 17>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> data: SdfSceneUniform;

/// SDF for ellipsoid with squash/stretch
fn sdf_ellipsoid(p: vec3<f32>, center: vec3<f32>, radius: f32,
                 stretch_dir: vec3<f32>, stretch: f32) -> f32 {
    let local_p = p - center;
    let compress = 1.0 / sqrt(stretch);
    let parallel = dot(local_p, stretch_dir) * stretch_dir;
    let perpendicular = local_p - parallel;
    let deformed = parallel * stretch + perpendicular * compress;
    return length(deformed) - radius;
}

/// SDF for a variable-radius cylinder (rubber band shape)
/// Thick at endpoints, thin in the middle
fn sdf_rubber_band(p: vec3<f32>, a: vec3<f32>, b: vec3<f32>, base_radius: f32) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);

    // Parabola: thick at both ends (h=0 and h=1), thin in middle (h=0.5)
    // This creates a symmetric rubber band shape
    let center_dist = abs(h - 0.5) * 2.0;  // 0 at center, 1 at ends

    // Thickness scaling: adjust these to change overall size while keeping ratio
    // Format: min_thickness + (max_thickness - min_thickness) * curve
    let min_thickness = 0.66;
    let max_thickness = 2.5;
    let thickness_curve = min_thickness + (max_thickness - min_thickness) * center_dist * center_dist;
    let radius = base_radius * thickness_curve;

    // Standard cylinder distance calculation with varying radius
    return length(pa - ba * h) - radius;
}

/// Smooth minimum for blending
fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h = max(k - abs(a - b), 0.0) / k;
    return min(a, b) - h * h * h * k * (1.0 / 6.0);
}

/// Add ripple distortion to a sphere (FIXED: much gentler)
fn apply_ripple(base_sdf: f32, p: vec3<f32>, center: vec3<f32>,
                phase: f32, amplitude: f32) -> f32 {
    // Only apply if amplitude is significant AND phase is recent
    if (amplitude < 0.01 || phase > 5.0) {
        return base_sdf;
    }

    let dist_from_center = length(p - center);

    // Much gentler wave (reduced frequency from 10.0 to 5.0)
    let wave_frequency = 5.0;
    let wave = sin(dist_from_center * wave_frequency - phase * 3.0);

    // Stronger falloff (decays faster, from 2.0 to 3.0)
    let falloff = exp(-phase * 3.0);

    // Much smaller amplitude (scale down by 10x!)
    let ripple_strength = wave * amplitude * falloff * 0.1;

    return base_sdf + ripple_strength;
}

/// Get color for a sphere surface point with infection gradient
fn get_sphere_color(
    sphere: SdfSphere,
    surface_point: vec3<f32>,
    sphere_idx: u32,
) -> vec4<f32> {
    // Quick exits
    if (sphere.infection_progress < 0.01) {
        return sphere.base_color;
    }

    if (sphere.infection_progress > 0.99) {
        return sphere.target_color;
    }

    var min_infection_dist = 999.0;

    // Find closest infection point
    for (var i = 0u; i < data.num_cylinders; i++) {
        let cyl = data.cylinders[i];

        // Check if this cylinder connects to this sphere
        var cyl_endpoint: vec3<f32>;
        var connects = false;

        if (cyl.node_a_idx == sphere_idx) {
            cyl_endpoint = cyl.start;
            connects = true;
        } else if (cyl.node_b_idx == sphere_idx) {
            cyl_endpoint = cyl.end;
            connects = true;
        }

        if (!connects) {
            continue;
        }

        // PROJECT the cylinder endpoint onto the sphere surface
        // This gives us the actual infection point on the sphere
        let to_endpoint = cyl_endpoint - sphere.center;
        let contact_point_on_surface = sphere.center + normalize(to_endpoint) * sphere.radius;

        // Now calculate distance along the sphere surface
        // Using arc length on the sphere: arc = radius * angle
        let to_contact = normalize(contact_point_on_surface - sphere.center);
        let to_surface = normalize(surface_point - sphere.center);

        // Angle in radians (acos of dot product)
        let cos_angle = dot(to_contact, to_surface);
        let angle = acos(clamp(cos_angle, -1.0, 1.0));

        // Arc length along sphere surface
        let surface_dist = angle * sphere.radius;

        min_infection_dist = min(min_infection_dist, surface_dist);
    }

    // Now use ACTUAL surface distance (not angular)
    // Ease-out cubic for snappy feel
    let eased_progress = 1.0 - pow(1.0 - sphere.infection_progress, 3.0);

    // Infection spreads across the surface
    // At progress=0: covers ~0.3 radius worth of surface
    // At progress=1: covers 2*PI*radius (entire sphere)
    let max_surface_distance = 3.14159 * sphere.radius;  // Half circumference
    let infection_reach = 0.2 * sphere.radius + eased_progress * max_surface_distance;

    // Smooth gradient at infection front
    let gradient_width = 0.3 * sphere.radius;
    let infection_amount = smoothstep(
        infection_reach + gradient_width,
        infection_reach - gradient_width,
        min_infection_dist
    );

    return mix(sphere.base_color, sphere.target_color, infection_amount);
}

/// Raymarch the entire scene
fn sdf_scene(p: vec3<f32>) -> vec3<f32> {  // Returns (distance, sphere_idx, is_sphere)
    var min_dist = 999999.0;
    var closest_sphere_idx = -1.0;
    var is_sphere = 0.0;

    // Check all spheres
    for (var i = 0u; i < data.num_spheres; i++) {
        let sphere = data.spheres[i];
        var d = sdf_ellipsoid(p, sphere.center, sphere.radius,
                              sphere.stretch_direction, sphere.stretch_factor);

        // Apply ripple
        d = apply_ripple(d, p, sphere.center, sphere.ripple_phase, sphere.ripple_amplitude);

        if (d < min_dist) {
            min_dist = d;
            closest_sphere_idx = f32(i);
            is_sphere = 1.0;
        }
    }

    // Check all cylinders and blend with spheres
    for (var i = 0u; i < data.num_cylinders; i++) {
        let cyl = data.cylinders[i];
        let d = sdf_rubber_band(p, cyl.start, cyl.end, cyl.radius);

        // Smooth blend
        let old_dist = min_dist;
        min_dist = smin(min_dist, d, 0.15);

        // If cylinder is now closest, mark it
        if (d < old_dist - 0.05) {
            closest_sphere_idx = f32(i);
            is_sphere = 0.0;  // It's a cylinder
        }
    }

    return vec3(min_dist, closest_sphere_idx, is_sphere);
}

fn raymarch(ro: vec3<f32>, rd: vec3<f32>) -> vec3<f32> {  // Returns (t, sphere_idx, is_sphere)
    var t = 0.0;
    var sphere_idx = -1.0;
    var is_sphere = 0.0;

    for (var i = 0; i < 128; i++) {
        let result = sdf_scene(ro + rd * t);
        let d = result.x;

        if (d < 0.001) {
            sphere_idx = result.y;
            is_sphere = result.z;
            return vec3(t, sphere_idx, is_sphere);
        }
        t += d * 0.9;
        if (t > 200.0) { break; }
    }
    return vec3(-1.0, sphere_idx, is_sphere);
}


fn normal_at(p: vec3<f32>) -> vec3<f32> {
    let e = 0.001;
    let dx = sdf_scene(vec3(p.x + e, p.y, p.z)).x
           - sdf_scene(vec3(p.x - e, p.y, p.z)).x;
    let dy = sdf_scene(vec3(p.x, p.y + e, p.z)).x
           - sdf_scene(vec3(p.x, p.y - e, p.z)).x;
    let dz = sdf_scene(vec3(p.x, p.y, p.z + e)).x
           - sdf_scene(vec3(p.x, p.y, p.z - e)).x;
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

    let result = raymarch(ro, rd);
    let t = result.x;
    let idx = i32(result.y);
    let is_sphere = result.z > 0.5;

    if (t > 0.0 && idx >= 0) {
        let hit = ro + rd * t;
        let n = normal_at(hit);

        // Lighting
        let light_dir = normalize(vec3(1.0, 1.0, 1.0));
        let diffuse = max(dot(n, light_dir), 0.0);
        let ambient = 0.3;
        let lighting = ambient + 0.7 * diffuse;

        // Get color based on what we hit
        var color: vec4<f32>;

        if (is_sphere) {
            // Hit a sphere - use infection gradient
            let sphere = data.spheres[idx];
            color = get_sphere_color(sphere, hit, u32(idx));
        } else {
            // Hit a cylinder
            let cyl = data.cylinders[idx];

            // Calculate position along cylinder for gradient
            let to_hit = hit - cyl.start;
            let cyl_dir = cyl.end - cyl.start;
            let t_cyl = clamp(dot(to_hit, cyl_dir) / dot(cyl_dir, cyl_dir), 0.0, 1.0);

            // Get endpoint sphere colors
            let sphere_a = data.spheres[cyl.node_a_idx];
            let sphere_b = data.spheres[cyl.node_b_idx];

            // Use infection-aware colors from spheres
            let color_a = get_sphere_color(sphere_a, cyl.start, cyl.node_a_idx);
            let color_b = get_sphere_color(sphere_b, cyl.end, cyl.node_b_idx);

            // Gradient along cylinder, lighter in middle
            let endpoint_color = mix(color_a, color_b, t_cyl);
            let lightness = 0.3 * (1.0 - 4.0 * (t_cyl - 0.5) * (t_cyl - 0.5)); // Parabola peaks at middle
            color = mix(endpoint_color, vec4(1.0, 1.0, 1.0, 1.0), lightness);
        }

        let clip = view.clip_from_world * vec4<f32>(hit, 1.0);
        let depth = clip.z / clip.w;

        return FragOut(vec4(color.rgb * lighting, color.a), depth);
    }

    discard;
}

