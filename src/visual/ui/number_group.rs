//! HUD number group specification for 7-segment display positioning and layout.
//!
//! This module provides data structures for declaratively defining groups of
//! digits and separators (tokens) that make up HUD displays like level numbers
//! and progress indicators.

/// Horizontal justification for a group of HUD tokens
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudJustify {
    /// Left-justify: first token starts at anchor point, extends right
    Left,
    /// Right-justify: last token ends at anchor point, extends left
    Right,
}

/// Anchor point for positioning a HUD group on the screen
#[derive(Clone, Copy, Debug)]
pub struct HudAnchor {
    /// Horizontal anchor position (0.0 = left edge, 1.0 = right edge)
    pub h: f32,
    /// Vertical anchor position (0.0 = bottom edge, 1.0 = top edge)
    pub v: f32,
    /// Padding from the edge as a fraction of screen bounds
    pub padding: f32,
}

/// Style parameters for rendering HUD digits and separators
#[derive(Clone, Copy, Debug)]
pub struct HudStyle {
    /// Scale multiplier for digit size in world space
    pub digit_scale: f32,
    /// Spacing between digits in digit-local units (multiplied by scale)
    pub digit_spacing: f32,
    /// Extra spacing around slash separators in digit-local units
    pub slash_spacing: f32,
}

impl Default for HudStyle {
    fn default() -> Self {
        Self {
            digit_scale: 0.25,  // Even smaller for better fit
            digit_spacing: 0.5, // More spacing between digits
            slash_spacing: 0.0, // More spacing around slash
        }
    }
}

/// A single token in a HUD display group
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudToken {
    /// A digit from 0 to 9
    Digit(u8),
    /// A forward slash separator
    Slash,
}

/// A group of HUD tokens positioned together
#[derive(Clone, Debug)]
pub struct HudGroup {
    /// Where to anchor this group on the screen
    pub anchor: HudAnchor,
    /// How to justify the tokens relative to the anchor
    pub justify: HudJustify,
    /// The sequence of tokens to display
    pub tokens: Vec<HudToken>,
}

/// Convert a number into a sequence of digit tokens.
///
/// # Examples
/// ```ignore
/// assert_eq!(tokens_for_number(0), vec![HudToken::Digit(0)]);
/// assert_eq!(tokens_for_number(42), vec![HudToken::Digit(4), HudToken::Digit(2)]);
/// assert_eq!(tokens_for_number(217), vec![HudToken::Digit(2), HudToken::Digit(1), HudToken::Digit(7)]);
/// ```
pub fn tokens_for_number(mut n: usize) -> Vec<HudToken> {
    if n == 0 {
        return vec![HudToken::Digit(0)];
    }
    let mut digits = Vec::new();
    while n > 0 {
        digits.push((n % 10) as u8);
        n /= 10;
    }
    digits.reverse();
    digits.into_iter().map(HudToken::Digit).collect()
}

/// Create a HUD group for displaying the current level number.
///
/// Positioned at the top-left with left justification.
/// Supports levels 1-217 (1-3 digits).
///
/// # Arguments
/// * `level` - The level number to display (1-217)
pub fn level_group(level: usize) -> HudGroup {
    HudGroup {
        anchor: HudAnchor {
            h: 0.0, // Left side - h=0 means left on screen!
            v: 0.99,
            padding: 0.05,
        },
        justify: HudJustify::Left,
        tokens: tokens_for_number(level),
    }
}

/// Create a HUD group for displaying progress as "found/total".
///
/// Positioned at the top-right with right justification.
/// Supports found: 0-95, total: 1-96 (1-2 digits each).
///
/// # Arguments
/// * `found` - Number of solutions found (0-95)
/// * `total` - Total number of solutions (1-96)
pub fn progress_group(found: usize, total: usize) -> HudGroup {
    let mut tokens = Vec::new();
    tokens.extend(tokens_for_number(found));
    tokens.push(HudToken::Slash);
    tokens.extend(tokens_for_number(total));
    HudGroup {
        anchor: HudAnchor {
            h: 1.0, // Right side - h=1 means right on screen!
            v: 0.99,
            padding: 0.05,
        },
        justify: HudJustify::Right,
        tokens,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokens_for_number() {
        assert_eq!(tokens_for_number(0), vec![HudToken::Digit(0)]);
        assert_eq!(tokens_for_number(7), vec![HudToken::Digit(7)]);
        assert_eq!(
            tokens_for_number(67),
            vec![HudToken::Digit(6), HudToken::Digit(7)]
        );
        assert_eq!(
            tokens_for_number(217),
            vec![HudToken::Digit(2), HudToken::Digit(1), HudToken::Digit(7)]
        );
    }

    #[test]
    fn test_level_group() {
        let group = level_group(67);
        assert_eq!(group.anchor.h, 0.0);
        assert_eq!(group.anchor.v, 1.0);
        assert!(matches!(group.justify, HudJustify::Left));
        assert_eq!(group.tokens.len(), 2);
    }

    #[test]
    fn test_progress_group() {
        let group = progress_group(5, 67);
        assert_eq!(group.anchor.h, 1.0);
        assert_eq!(group.anchor.v, 1.0);
        assert!(matches!(group.justify, HudJustify::Right));

        // Should be: 5, /, 1, 2
        assert_eq!(group.tokens.len(), 4);
        assert!(matches!(group.tokens[0], HudToken::Digit(5)));
        assert!(matches!(group.tokens[1], HudToken::Slash));
        assert!(matches!(group.tokens[2], HudToken::Digit(6)));
        assert!(matches!(group.tokens[3], HudToken::Digit(7)));
    }
}
