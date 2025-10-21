// ============================================================================
// EASING FUNCTIONS for smooth animations
// ============================================================================

use bevy::math::{Vec3, Vec4};

/// Ease-in-out cubic: slow at start and end, fast in the middle (RECOMMENDED for HSV)
/// Perfect for organic color transitions - smooth S-curve
pub fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// Ease-out cubic: fast at start, decelerates at end
/// Good for "arriving" animations - color change is immediately visible
pub fn ease_out_cubic(t: f32) -> f32 {
    let x = 1.0 - t;
    1.0 - x * x * x
}

/// Ease-in cubic: slow at start, fast at end
/// Good for "launching" animations
#[allow(dead_code)]
pub fn ease_in_cubic(t: f32) -> f32 {
    t * t * t
}

/// Ease-out quadratic: fast at start, decelerates (gentler than cubic)
#[allow(dead_code)]
pub fn ease_out_quad(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

/// Linear: no easing, constant speed
#[allow(dead_code)]
pub fn linear(t: f32) -> f32 {
    t
}

// ============================================================================
// HSV COLOR SPACE CONVERSION - For smooth color transitions around the wheel
// ============================================================================

/// Convert RGB to HSV (Hue, Saturation, Value)
pub fn rgb_to_hsv(color: Vec4) -> Vec3 {
    let r = color.x;
    let g = color.y;
    let b = color.z;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    // Hue
    let h = if delta < 0.00001 {
        0.0
    } else if (max - r).abs() < 0.00001 {
        ((g - b) / delta) % 6.0
    } else if (max - g).abs() < 0.00001 {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    };
    let h = (h / 6.0).rem_euclid(1.0); // Normalize to [0, 1]

    // Saturation
    let s = if max < 0.00001 { 0.0 } else { delta / max };

    // Value
    let v = max;

    Vec3::new(h, s, v)
}

/// Convert HSV back to RGB
pub fn hsv_to_rgb(hsv: Vec3) -> Vec4 {
    let h = hsv.x * 6.0; // Hue in [0, 6]
    let s = hsv.y;
    let v = hsv.z;

    let c = v * s;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h < 1.0 {
        (c, x, 0.0)
    } else if h < 2.0 {
        (x, c, 0.0)
    } else if h < 3.0 {
        (0.0, c, x)
    } else if h < 4.0 {
        (0.0, x, c)
    } else if h < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    Vec4::new(r + m, g + m, b + m, 1.0)
}

/// Lerp two colors in HSV space (takes shortest path around hue wheel)
pub fn lerp_hsv(color_a: Vec4, color_b: Vec4, t: f32) -> Vec4 {
    let hsv_a = rgb_to_hsv(color_a);
    let hsv_b = rgb_to_hsv(color_b);

    // Handle hue wrapping (shortest path around color wheel)
    let mut hue_a = hsv_a.x;
    let mut hue_b = hsv_b.x;

    // If hues are more than 180° apart, wrap around
    if (hue_b - hue_a).abs() > 0.5 {
        if hue_a < hue_b {
            hue_a += 1.0;
        } else {
            hue_b += 1.0;
        }
    }

    // Mix in HSV space
    let mixed_hue = (hue_a + (hue_b - hue_a) * t).rem_euclid(1.0);
    let mixed_sat = hsv_a.y + (hsv_b.y - hsv_a.y) * t;
    let mixed_val = hsv_a.z + (hsv_b.z - hsv_a.z) * t;

    hsv_to_rgb(Vec3::new(mixed_hue, mixed_sat, mixed_val))
}
