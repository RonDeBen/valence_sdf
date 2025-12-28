use bevy::prelude::*;

mod camera;
mod game;
mod graph;
mod input;
mod visual;

use camera::CameraPlugin;
use input::InputPlugin;
use visual::experiment::ExperimentMaterialPlugin;
use visual::sdf::material::SdfMaterialPlugin;

use crate::visual::plugin::GraphPlugin;

// 🎨 SCENE SELECTOR - Change this to switch between modes!
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SceneMode {
    /// The main graph puzzle visualization
    GraphVisualization,
    /// Experimental shader playground (hot reloadable)
    Experiment,
}

const ACTIVE_SCENE: SceneMode = SceneMode::Experiment;

fn main() {
    let mut app = App::new();

    // Always add these base plugins
    app.add_plugins(DefaultPlugins)
        .add_plugins(CameraPlugin)
        .add_systems(Startup, setup_lighting);

    // Add scene-specific plugins based on mode
    match ACTIVE_SCENE {
        SceneMode::GraphVisualization => {
            info!("🎮 Starting in Graph Visualization mode");
            app.add_plugins(InputPlugin)
                .add_plugins(SdfMaterialPlugin)
                .add_plugins(GraphPlugin);
        }
        SceneMode::Experiment => {
            info!("🧪 Starting in Experiment mode - Hot reload enabled!");
            info!("   Edit assets/shaders/experiment.wgsl and save to see changes instantly");
            app.add_plugins(ExperimentMaterialPlugin);
        }
    }

    app.run();
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
