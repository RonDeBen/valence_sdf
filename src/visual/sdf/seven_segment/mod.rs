pub mod digit;
pub mod material;

// Re-export commonly used types
pub use digit::Digit;
pub use material::{
    HudInstance, MAX_HUD_INSTANCES, SevenSegmentMaterial, SevenSegmentMaterialPlugin,
};
