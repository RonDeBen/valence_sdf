// camera.rs

use bevy::camera::{ScalingMode, Viewport};
use bevy::prelude::*;
use bevy::window::WindowResized;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameCamera>()
            .add_systems(Startup, setup_camera)
            .add_systems(Update, update_camera_viewport);
    }
}

// 🔧 FIXED ASPECT RATIO - This never changes!
// Bottom-left origin: (0, 0) to (GAME_WIDTH, GAME_HEIGHT)
const GAME_HEIGHT: f32 = 8.0; // World units
const GAME_ASPECT_RATIO: f32 = 9.0 / 16.0; // Portrait
const GAME_WIDTH: f32 = GAME_HEIGHT * GAME_ASPECT_RATIO; // 4.5 world units

#[derive(Resource)]
pub struct GameCamera {
    pub bounds: CameraBounds,
    pub entity: Option<Entity>,
}

#[derive(Debug, Clone, Copy)]
pub struct CameraBounds {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
}

impl Default for GameCamera {
    fn default() -> Self {
        Self {
            bounds: CameraBounds::from_fixed_aspect(),
            entity: None,
        }
    }
}

impl CameraBounds {
    /// Create bounds with FIXED aspect ratio, bottom-left origin at (0, 0)
    pub fn from_fixed_aspect() -> Self {
        Self {
            left: 0.0,
            right: GAME_WIDTH,
            bottom: 0.0,
            top: GAME_HEIGHT,
        }
    }

    pub fn width(&self) -> f32 {
        self.right - self.left
    }

    pub fn height(&self) -> f32 {
        self.top - self.bottom
    }

    /// CSS-style anchor positioning in XY plane
    pub fn anchor(&self, horizontal: f32, vertical: f32, padding: f32) -> Vec3 {
        let padded_left = self.left + self.width() * padding;
        let padded_right = self.right - self.width() * padding;
        let padded_bottom = self.bottom + self.height() * padding;
        let padded_top = self.top - self.height() * padding;

        let x = padded_left + (padded_right - padded_left) * horizontal;
        let y = padded_bottom + (padded_top - padded_bottom) * vertical;

        Vec3::new(x, y, 0.0)
    }

    /// Get a rectangular region
    pub fn region(
        &self,
        h_start: f32,
        h_end: f32,
        v_start: f32,
        v_end: f32,
        padding: f32,
    ) -> CameraBounds {
        let padded_width = self.width() * (1.0 - 2.0 * padding);
        let padded_height = self.height() * (1.0 - 2.0 * padding);
        let padded_left = self.left + self.width() * padding;
        let padded_bottom = self.bottom + self.height() * padding;

        CameraBounds {
            left: padded_left + padded_width * h_start,
            right: padded_left + padded_width * h_end,
            bottom: padded_bottom + padded_height * v_start,
            top: padded_bottom + padded_height * v_end,
        }
    }
}

#[derive(Component)]
pub struct MainCamera;

fn setup_camera(mut commands: Commands, game_camera: Res<GameCamera>) {
    let projection = Projection::Orthographic(OrthographicProjection {
        scaling_mode: ScalingMode::FixedVertical {
            viewport_height: GAME_HEIGHT,
        },
        near: -1000.0,
        far: 1000.0,
        ..OrthographicProjection::default_3d()
    });

    // Position camera at center of game area, looking down -Z onto XY plane
    let cx = GAME_WIDTH * 0.5;
    let cy = GAME_HEIGHT * 0.5;

    commands.spawn((
        Camera3d::default(),
        projection,
        Transform::from_xyz(cx, cy, 10.0).looking_at(Vec3::new(cx, cy, 0.0), Vec3::Y),
        MainCamera,
    ));

    info!("📷 Camera setup: XY plane, bottom-left origin (0,0)");
    info!("   Game bounds: ({:.2}, {:.2}) to ({:.2}, {:.2})",
        game_camera.bounds.left, game_camera.bounds.bottom,
        game_camera.bounds.right, game_camera.bounds.top
    );
}

/// Update camera viewport to maintain aspect ratio with letterboxing
fn update_camera_viewport(
    mut cameras: Query<&mut Camera, With<MainCamera>>,
    windows: Query<&Window>,
    mut resize_events: MessageReader<WindowResized>, // 🔧 This is the idiomatic way!
) {
    // 🔧 Only runs when there's an actual resize event
    for _event in resize_events.read() {
        let Ok(window) = windows.single() else {
            continue;
        };

        let Ok(mut camera) = cameras.single_mut() else {
            continue;
        };

        let window_width = window.physical_width();
        let window_height = window.physical_height();
        let window_aspect = window_width as f32 / window_height as f32;

        // Calculate viewport to maintain game aspect ratio
        let (viewport_width, viewport_height, x_offset, y_offset) =
            if window_aspect > GAME_ASPECT_RATIO {
                // Window is wider - pillarboxing (black bars on sides)
                let viewport_width = (window_height as f32 * GAME_ASPECT_RATIO) as u32;
                let x_offset = (window_width - viewport_width) / 2;
                (viewport_width, window_height, x_offset, 0)
            } else {
                // Window is taller - letterboxing (black bars top/bottom)
                let viewport_height = (window_width as f32 / GAME_ASPECT_RATIO) as u32;
                let y_offset = (window_height - viewport_height) / 2;
                (window_width, viewport_height, 0, y_offset)
            };

        camera.viewport = Some(Viewport {
            physical_position: UVec2::new(x_offset, y_offset),
            physical_size: UVec2::new(viewport_width, viewport_height),
            ..default()
        });

        info!(
            "📐 Viewport updated: {}x{} at ({}, {})",
            viewport_width, viewport_height, x_offset, y_offset
        );
        info!(
            "   Window: {}x{} (aspect {:.2}), Game aspect: {:.2}",
            window_width, window_height, window_aspect, GAME_ASPECT_RATIO
        );
    }
}
