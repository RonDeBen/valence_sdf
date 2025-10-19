use bevy::camera::ScalingMode;
use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameCamera>()
            .add_systems(Startup, setup_camera)
            .add_systems(Update, update_camera_resource);
    }
}

#[derive(Resource)]
pub struct GameCamera {
    pub scale: f32,
    pub aspect_ratio: f32,
    pub bounds: CameraBounds,
}

#[derive(Debug, Clone)]
pub struct CameraBounds {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
}

impl Default for GameCamera {
    fn default() -> Self {
        let scale = 8.0;
        let aspect_ratio = 16.0 / 9.0;

        Self {
            scale,
            aspect_ratio,
            bounds: CameraBounds::from_scale_and_aspect(scale, aspect_ratio),
        }
    }
}

impl CameraBounds {
    pub fn from_scale_and_aspect(scale: f32, aspect_ratio: f32) -> Self {
        // For orthographic, scale determines the vertical view
        let half_height = scale * 0.5;
        let half_width = half_height * aspect_ratio;

        Self {
            left: -half_width,
            right: half_width,
            bottom: -half_height,
            top: half_height,
        }
    }

    /// Get width of visible area
    pub fn width(&self) -> f32 {
        self.right - self.left
    }

    /// Get height of visible area
    pub fn height(&self) -> f32 {
        self.top - self.bottom
    }

    /// Calculate position with percentage-based padding
    /// For example: position_with_padding(0.5, 0.8, 0.1)
    /// puts something at 50% horizontal, 80% vertical, with 10% padding
    pub fn position_with_padding(
        &self,
        horizontal_percent: f32, // 0.0 = left, 1.0 = right
        vertical_percent: f32,   // 0.0 = bottom, 1.0 = top
        padding_percent: f32,    // Amount to inset from edges
    ) -> Vec3 {
        let padded_left = self.left + self.width() * padding_percent;
        let padded_right = self.right - self.width() * padding_percent;
        let padded_bottom = self.bottom + self.height() * padding_percent;
        let padded_top = self.top - self.height() * padding_percent;

        let x = padded_left + (padded_right - padded_left) * horizontal_percent;
        let z = padded_bottom + (padded_top - padded_bottom) * vertical_percent;

        Vec3::new(x, 0.0, z)
    }

    /// Calculate a region
    pub fn region(
        &self,
        horizontal_range: (f32, f32),
        vertical_range: (f32, f32),
        padding_percent: f32,
    ) -> CameraBounds {
        let padded_width = self.width() * (1.0 - 2.0 * padding_percent);
        let padded_height = self.height() * (1.0 - 2.0 * padding_percent);
        let padded_left = self.left + self.width() * padding_percent;
        let padded_bottom = self.bottom + self.height() * padding_percent;

        CameraBounds {
            left: padded_left + padded_width * horizontal_range.0,
            right: padded_left + padded_width * horizontal_range.1,
            bottom: padded_bottom + padded_height * vertical_range.0,
            top: padded_bottom + padded_height * vertical_range.1,
        }
    }
}

#[derive(Component)]
pub struct MainCamera;

/// Setup a top-down orthographic camera looking at the XZ plane
/// 
/// Coordinate System (right-handed, Y-up):
/// ```
///        Y (height)
///        ↑
///        |  
///   ----+---→ X (right on screen)
///       /
///      ↗ Z (up on screen when viewed from above)
/// ```
/// 
/// Camera looks down from +Y axis, with +Z pointing up on screen
/// Game board is in the XZ plane (y=0)
fn setup_camera(mut commands: Commands, game_camera: Res<GameCamera>) {
    let projection = Projection::Orthographic(OrthographicProjection {
        scaling_mode: ScalingMode::FixedVertical { viewport_height: game_camera.scale },
        near: 0.0,
        far: 1000.0,
        ..OrthographicProjection::default_3d()
    });
    commands.spawn((
        Camera3d::default(),
        projection,
        // Camera 10 units above origin, looking down at origin
        // Vec3::Z means +Z direction points "up" on screen (towards top of monitor)
        Transform::from_xyz(0.0, 10.0, 0.0).looking_at(Vec3::ZERO, Vec3::Z),
        MainCamera,
    ));
}

/// Update camera resource when window is resized
fn update_camera_resource(
    mut game_camera: ResMut<GameCamera>,
    windows: Query<&Window>,
) {
    if let Ok(window) = windows.single() {
        let new_aspect = window.width() / window.height();

        // Only update if aspect ratio changed
        if (new_aspect - game_camera.aspect_ratio).abs() > 0.01 {
            game_camera.aspect_ratio = new_aspect;

            game_camera.bounds =
                CameraBounds::from_scale_and_aspect(game_camera.scale, game_camera.aspect_ratio);

            println!("Camera bounds updated: {:?}", game_camera.bounds);
        }
    }
}
