//! HUD instance builder - converts HUD groups to shader instances.

use bevy::prelude::*;

use crate::camera::CameraBounds;
use crate::visual::sdf::seven_segment::{Digit, HudInstance};

use super::number_group::*;

/// Convert anchor coordinates to world position
/// 
/// Returns Vec2 in world XY plane where the HUD element should be positioned
pub fn anchor_world(bounds: &CameraBounds, anchor: HudAnchor) -> Vec2 {
    let w = bounds.width();
    let h = bounds.height();
    
    // Calculate positions with padding
    let x0 = bounds.left + w * anchor.padding;
    let x1 = bounds.right - w * anchor.padding;
    let y0 = bounds.bottom + h * anchor.padding;
    let y1 = bounds.top - h * anchor.padding;
    
    // Interpolate based on anchor (h: 0=left, 1=right, v: 0=bottom, 1=top)
    let x = x0 + (x1 - x0) * anchor.h;
    let y = y0 + (y1 - y0) * anchor.v;
    
    // Return world position directly - shader uses: p = vec2(world_x, world_y)
    Vec2::new(x, y)
}

/// Calculate total width of a token group
fn group_width(tokens: &[HudToken], digit_w: f32, gap: f32, slash_extra: f32) -> f32 {
    let mut w = 0.0;
    for (i, token) in tokens.iter().enumerate() {
        if i > 0 {
            w += gap;
        }
        w += digit_w;
        if matches!(token, HudToken::Slash) {
            w += slash_extra;
        }
    }
    w
}

/// Build HUD instances from a number group
///
/// Converts a `HudGroup` into a list of `HudInstance` structs for the shader.
/// 
/// Position convention: all positions are CENTERS of tokens in world XZ space.
pub fn build_instances_for_group(
    bounds: &CameraBounds,
    group: &HudGroup,
    style: HudStyle,
    out: &mut Vec<HudInstance>,
) {
    if group.tokens.is_empty() {
        return;
    }

    // Get anchor position in world space
    let anchor = anchor_world(bounds, group.anchor);
    
    // Calculate dimensions
    let digit_w = style.digit_scale;
    let gap = style.digit_spacing * digit_w;
    let slash_extra = style.slash_spacing * digit_w;
    let total_w = group_width(&group.tokens, digit_w, gap, slash_extra);

    // Calculate starting X based on justification
    // - Left: anchor is at CENTER of first token
    // - Right: anchor is at CENTER of last token
    let start_x = match group.justify {
        HudJustify::Left => anchor.x,
        HudJustify::Right => anchor.x - total_w + digit_w,
    };

    // Place each token, treating x as CENTER
    let mut x = start_x;
    for token in &group.tokens {
        let (kind, mask) = match token {
            HudToken::Digit(d) => {
                // Convert usize to Digit enum
                let digit = match d {
                    0 => Digit::Zero,
                    1 => Digit::One,
                    2 => Digit::Two,
                    3 => Digit::Three,
                    4 => Digit::Four,
                    5 => Digit::Five,
                    6 => Digit::Six,
                    7 => Digit::Seven,
                    8 => Digit::Eight,
                    9 => Digit::Nine,
                    _ => Digit::Eight, // Default to 8 for invalid digits
                };
                (0u32, digit.mask() as u32)
            }
            HudToken::Slash => (1u32, 0u32), // Slash doesn't use mask
        };

        out.push(HudInstance {
            kind,
            mask,
            from_mask: mask,  // Initially, no transition (from == to)
            transition_progress: 1.0,  // Fully transitioned
            pos: Vec2::new(x, anchor.y),
            scale: digit_w,
            _pad1: 0,
            _pad2: 0,
            _pad3: 0,
            _pad4: 0,
        });

        // Move to next token
        x += digit_w + gap;
        if matches!(token, HudToken::Slash) {
            x += slash_extra;
        }
    }
}
