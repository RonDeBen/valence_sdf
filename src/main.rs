use bevy::prelude::*;

mod camera;
mod game;
mod graph;
mod input;
mod visual;

use camera::CameraPlugin;
use input::InputPlugin;
use visual::sdf::material::SdfMaterialPlugin;

use crate::visual::plugin::GraphPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CameraPlugin)
        .add_plugins(InputPlugin)
        .add_plugins(SdfMaterialPlugin)
        .add_plugins(GraphPlugin)
        .add_systems(Startup, setup_lighting)
        // .add_systems(Startup, spawn_sdf_screen)  // Temporarily disabled to see grid
        .run();
}

fn setup_lighting(mut commands: Commands) {
    // Directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 200.0,
        affects_lightmapped_meshes: false,
    });
}
