pub mod edges;
pub mod interactions;
pub mod nodes;
pub mod physics;
pub mod plugin;
pub mod sdf;
pub mod setup;
pub mod ui;
pub mod utils;

// Public exports for SDF materials (used in GraphVisualization mode)
pub use sdf::material::{SceneMaterialHandle, SdfSceneMaterial};
