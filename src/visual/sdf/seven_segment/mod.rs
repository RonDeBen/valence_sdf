//! 7-segment display system for HUD rendering with animated transitions.
//!
//! This module provides a complete 7-segment display system separate from the glyph-based
//! number rendering in the `numbers` module. It includes:
//!
//! - **Digit representation** (`digit.rs`): `Digit` enum with mask encoding/decoding
//! - **Transition logic** (`transitions.rs`): Flow computation for animated transitions
//! - **Material & shader** (`material.rs`): Bevy material for rendering the displays
//!
//! The system uses a unified rendering approach where one large plane renders all HUD
//! elements from an array of `HudInstance` structs, with each digit animating independently.

pub mod digit;
pub mod material;
pub mod transitions;

// Re-export commonly used types
pub use digit::Digit;
pub use material::{
    HudInstance, SevenSegmentMaterial, SevenSegmentMaterialPlugin, MAX_HUD_INSTANCES,
};
