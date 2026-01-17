use bevy::prelude::*;

mod camera;
mod game;
mod graph;
mod input;
mod visual;

use bevy::window::WindowResolution;
use camera::CameraPlugin;
use input::InputPlugin;
use visual::sdf::material::SdfMaterialPlugin;
use visual::sdf::seven_segment::SevenSegmentMaterialPlugin;

use crate::visual::plugin::GraphPlugin;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Valence SDF".into(),
            resolution: WindowResolution::new(1080, 1920),
            resizable: true,
            ..default()
        }),
        ..default()
    }))
    .add_plugins(CameraPlugin)
    .add_plugins(InputPlugin)
    .add_plugins(SdfMaterialPlugin)
    .add_plugins(SevenSegmentMaterialPlugin)
    .add_plugins(GraphPlugin);

    app.run();
}
