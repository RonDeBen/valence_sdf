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

/// SDF for a regular cylinder (constant radius)
fn sdf_cylinder(p: vec3<f32>, a: vec3<f32>, b: vec3<f32>, radius: f32) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h) - radius;
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

/// Combined ripple + pop effect - MOBILE OPTIMIZED with gentle bounce
fn apply_ripple(base_sdf: f32, p: vec3<f32>, center: vec3<f32>,
                phase: f32, amplitude: f32) -> f32 {
    // Shorter duration - about 3.5 seconds total
    if (amplitude < 0.01 || phase > 10.0) {
        return base_sdf;
    }

    let dist_from_center = length(p - center);

    // === PART 1: Gentle Bounce Pop Effect ===
    // One main expansion, then 1-2 gentle bounces back to normal
    let bounce_frequency = 0.6;  // Much slower - just ~1.5 total bounces over 5 seconds
    let damping = 0.5;           // How quickly it settles (0.5 = gentle settling)

    // Exponential decay envelope (gradual energy loss)
    let envelope = exp(-phase * damping);

    // Oscillation (the gentle bouncing in and out)
    let oscillation = sin(phase * bounce_frequency * 6.28318);  // 6.28318 = 2*PI

    // Combined: gentle bouncing that settles gradually
    let pop_strength = oscillation * envelope * amplitude * 0.22;  // 22% max expansion

    // === PART 2: Traveling Wave Ripples - MORE PRONOUNCED ===
    // Multiple wave cycles, stronger amplitude
    let wave_frequency = 5.0;  // More ripples (was 4.0) - more cycles visible
    let wave_speed = 3.0;      // Medium speed (was 2.5)

    // Wave equation - now with stronger amplitude
    let wave = sin(dist_from_center * wave_frequency - phase * wave_speed);

    // Slower decay so waves travel farther and stay visible longer
    let time_decay = exp(-phase * 0.6);  // Even slower (was 0.8)
    let distance_decay = exp(-dist_from_center * 0.7);  // Travels farther (was 0.8)

    let wave_falloff = time_decay * distance_decay;
    let wave_strength = wave * amplitude * wave_falloff * 0.28;  // BIGGER waves (was 0.2)

    // === COMBINE BOTH EFFECTS ===
    // Pop makes the whole sphere bounce in/out like a basketball
    // Waves add traveling ripples on top (more pronounced)
    return base_sdf - pop_strength + wave_strength;
}

/// Radial pop effect - sphere briefly expands then contracts (ALTERNATIVE VERSION)
/// To use this: swap the function names (rename apply_ripple → apply_ripple_wave, this → apply_ripple)
fn apply_ripple_pop(base_sdf: f32, p: vec3<f32>, center: vec3<f32>,
                    phase: f32, amplitude: f32) -> f32 {
    if (amplitude < 0.01 || phase > 3.0) {
        return base_sdf;
    }

    // Ease-out: fast expansion, slow return
    let t = phase / 3.0;  // Normalize to 0-1 over 3 seconds (longer duration)
    let expansion = sin(t * 3.14159) * amplitude;  // Smooth pop

    // Time decay
    let time_decay = 1.0 - t;

    // Uniform expansion (makes whole sphere bigger then smaller)
    // Increased from 0.05 to 0.12 for more visible effect
    let size_change = expansion * time_decay * 0.12;

    return base_sdf - size_change;
}

fn raymarch_thickness(ro: vec3<f32>, rd: vec3<f32>, max_thickness: f32) -> f32 {
    var t = 0.01;  // Start slightly inside the surface
    var thickness = 0.0;

    // March until we exit the object or reach max distance
    for (var i = 0; i < 32; i++) {  // Fewer iterations needed for thickness
        let p = ro + rd * t;
        let d = sdf_scene(p).x;

        // If we've exited (SDF becomes positive), we're done
        if (d > 0.001) {
            break;
        }

        // Still inside - keep marching
        thickness = t;
        t += max(abs(d), 0.01);  // Step by the distance (inside = negative)

        if (t > max_thickness) {
            thickness = max_thickness;
            break;
        }
    }

    return thickness;
}

/// Convert thickness to opacity (Beer-Lambert law)
/// Thin = transparent, thick = opaque
fn thickness_to_opacity(thickness: f32, density: f32) -> f32 {
    // Beer-Lambert law: I = I₀ * e^(-density * thickness)
    // Opacity = 1 - transmission
    let transmission = exp(-density * thickness);
    return 1.0 - transmission;
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
        // d = apply_ripple_pop(d, p, sphere.center, sphere.ripple_phase, sphere.ripple_amplitude);

        if (d < min_dist) {
            min_dist = d;
            closest_sphere_idx = f32(i);
            is_sphere = 1.0;
        }
    }

    // Check all cylinders and blend with spheres
    for (var i = 0u; i < data.num_cylinders; i++) {
        let cyl = data.cylinders[i];
        
        // Preview edges (where node_a_idx == node_b_idx) use regular cylinder
        // Regular edges use rubber band shape
        var d: f32;
        if (cyl.node_a_idx == cyl.node_b_idx) {
            // Preview edge: constant radius (no thick blob at cursor)
            d = sdf_cylinder(p, cyl.start, cyl.end, cyl.radius);
        } else {
            // Regular edge: rubber band shape (thick at ends, thin in middle)
            d = sdf_rubber_band(p, cyl.start, cyl.end, cyl.radius);
        }

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

/// Convert RGB to HSV for color wheel blending
fn rgb_to_hsv(c: vec3<f32>) -> vec3<f32> {
    let K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    let p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    let q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));
    let d = q.x - min(q.w, q.y);
    let e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

/// Convert HSV back to RGB
fn hsv_to_rgb(c: vec3<f32>) -> vec3<f32> {
    let K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, vec3(0.0), vec3(1.0)), c.y);
}

/// Mix two colors in HSV space (takes shortest path around color wheel)
fn mix_hsv(color_a: vec3<f32>, color_b: vec3<f32>, t: f32) -> vec3<f32> {
    let hsv_a = rgb_to_hsv(color_a);
    let hsv_b = rgb_to_hsv(color_b);

    // Handle hue wrapping (shortest path around color wheel)
    var hue_a = hsv_a.x;
    var hue_b = hsv_b.x;

    // If hues are more than 180° apart, wrap around
    if (abs(hue_b - hue_a) > 0.5) {
        if (hue_a < hue_b) {
            hue_a += 1.0;
        } else {
            hue_b += 1.0;
        }
    }

    // Mix in HSV space
    let mixed_hue = fract(mix(hue_a, hue_b, t));  // Wrap back to [0, 1]
    let mixed_sat = mix(hsv_a.y, hsv_b.y, t);
    let mixed_val = mix(hsv_a.z, hsv_b.z, t);

    return hsv_to_rgb(vec3(mixed_hue, mixed_sat, mixed_val));
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

        // === LIGHTING ===
        let light_dir = normalize(vec3(1.0, 1.0, 1.0));
        let view_dir = normalize(cam - hit);

        // Cel-shaded diffuse
        let diffuse_raw = max(dot(n, light_dir), 0.0);
        let diffuse_stepped = smoothstep(0.3, 0.35, diffuse_raw);

        // Sharp specular highlight
        let half_dir = normalize(light_dir + view_dir);
        let spec_raw = pow(max(dot(n, half_dir), 0.0), 64.0);
        let specular = step(0.8, spec_raw) * 1.5;

        let lit = mix(0.6, 1.2, diffuse_stepped);
        let lighting = lit + specular;

        // === COLOR ===
        var base_color: vec4<f32>;
        var position_along_cylinder: f32 = 0.5;
        var glow: f32 = 0.0;  // Track glow intensity (multi-purpose)

        if (is_sphere) {
            let sphere = data.spheres[idx];
            base_color = get_sphere_color(sphere, hit, u32(idx));
            glow = sphere.spike_amount;  // Get glow from spike_amount field
        } else {
            let cyl = data.cylinders[idx];
            let to_hit = hit - cyl.start;
            let cyl_dir = cyl.end - cyl.start;
            let t_cyl = clamp(dot(to_hit, cyl_dir) / dot(cyl_dir, cyl_dir), 0.0, 1.0);

            position_along_cylinder = t_cyl;

            let sphere_a = data.spheres[cyl.node_a_idx];
            let sphere_b = data.spheres[cyl.node_b_idx];

            // Get animated colors and blend in HSV space
            let color_a = get_sphere_color(sphere_a, cyl.start, cyl.node_a_idx);
            let color_b = get_sphere_color(sphere_b, cyl.end, cyl.node_b_idx);
            let mixed_color = mix_hsv(color_a.rgb, color_b.rgb, t_cyl);
            base_color = vec4(mixed_color, 1.0);
        }

        // === OPACITY ===
        var opacity: f32;
        if (is_sphere) {
            opacity = 0.95;  // Nodes mostly solid
        } else {
            // Cylinders: solid at ends, transparent in middle
            let dist_from_center = abs(position_along_cylinder - 0.5) * 2.0;
            opacity = mix(0.5, 0.95, dist_from_center * dist_from_center);
        }

        // === COLOR BOOST ===
        let saturation_boost = 1.6;
        let brightness_boost = 1.4;

        let gray = dot(base_color.rgb, vec3(0.299, 0.587, 0.114));
        let boosted_color = mix(vec3(gray), base_color.rgb, saturation_boost) * brightness_boost;
        let clamped_color = clamp(boosted_color, vec3(0.0), vec3(1.0));

        // === SUBSURFACE GLOW (cylinders only) ===
        var subsurface_glow = vec3(0.0);
        if (!is_sphere) {
            // Middle of cylinder glows when backlit
            let dist_from_center = abs(position_along_cylinder - 0.5) * 2.0;
            let glow_amount = (1.0 - dist_from_center) * 0.5;  // Peaks at center
            let backlight = max(dot(-n, light_dir), 0.0);
            subsurface_glow = clamped_color * glow_amount * backlight;
        }

        // === FINAL COLOR ===
        let surface_color = clamped_color * lighting;
        let final_color = surface_color + subsurface_glow;

        // === RIM LIGHT ===
        let fresnel_raw = pow(1.0 - abs(dot(n, view_dir)), 2.0);
        let fresnel_stepped = smoothstep(0.65, 0.75, fresnel_raw);

        let distance_to_cam = length(hit - cam);
        let distance_fade = 1.0 - smoothstep(5.0, 15.0, distance_to_cam);

        let rim_strength = 0.6;
        let rim_glow = clamped_color * fresnel_stepped * rim_strength * distance_fade;
        var with_rim = final_color + rim_glow;
        
        // === GLOW EFFECT (additive emission) ===
        // Add glow AFTER all lighting so it's visible on any color!
        if (is_sphere && glow > 0.01) {
            // Additive glow in the node's own color
            let emission = clamped_color * glow * 0.6;
            with_rim = with_rim + emission;  // Don't clamp - let it glow!
        }

        let clip = view.clip_from_world * vec4<f32>(hit, 1.0);
        let depth = clip.z / clip.w;

        return FragOut(
            vec4(with_rim, opacity),
            depth
        );
    }

    discard;
}

