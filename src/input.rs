use bevy::input::touch::{TouchInput, TouchPhase};
use bevy::prelude::*;
use bevy::window::CursorMoved;

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorPos>()
            .add_message::<PointerEvent>()
            .add_systems(Update, (track_cursor_pos, collect_pointer_events));
    }
}

#[derive(Message, Debug, Clone)]
pub struct PointerEvent {
    /// Window (logical) coordinates: pixels from bottom-left
    pub position: Vec2,
    pub event_type: PointerEventType,
    /// 0 = mouse, >0 = touch id (handy later if you add multi-touch)
    pub id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerEventType {
    Down,
    Move,
    Up,
}

impl PointerEvent {
    /// Convert window coords to world space using a camera
    pub fn to_world_position(
        &self,
        camera: &Camera,
        camera_transform: &GlobalTransform,
    ) -> Option<Vec3> {
        camera
            .viewport_to_world(camera_transform, self.position)
            .ok()
            .map(|ray| {
                // Example: intersect y=0 plane (top-down ortho)
                let t = -ray.origin.y / ray.direction.y;
                ray.origin + ray.direction * t
            })
    }
}

#[derive(Resource, Default, Debug, Clone, Copy)]
struct CursorPos(pub Option<Vec2>);

fn track_cursor_pos(mut ev_cursor: MessageReader<CursorMoved>, mut pos: ResMut<CursorPos>) {
    for e in ev_cursor.read() {
        // last event wins; bottom-left origin already
        pos.0 = Some(e.position);
    }
}

fn collect_pointer_events(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    cursor: Res<CursorPos>,
    mut touch_events: MessageReader<TouchInput>,
    mut out: MessageWriter<PointerEvent>,
) {
    if let Some(p) = cursor.0 {
        if mouse_buttons.just_pressed(MouseButton::Left) {
            out.write(PointerEvent {
                position: p,
                event_type: PointerEventType::Down,
                id: 0,
            });
        }
        if mouse_buttons.pressed(MouseButton::Left) {
            out.write(PointerEvent {
                position: p,
                event_type: PointerEventType::Move,
                id: 0,
            });
        }
        if mouse_buttons.just_released(MouseButton::Left) {
            out.write(PointerEvent {
                position: p,
                event_type: PointerEventType::Up,
                id: 0,
            });
        }
    }

    for ev in touch_events.read() {
        let event_type = match ev.phase {
            TouchPhase::Started => PointerEventType::Down,
            TouchPhase::Moved => PointerEventType::Move,
            TouchPhase::Ended | TouchPhase::Canceled => PointerEventType::Up,
        };
        out.write(PointerEvent {
            position: ev.position,
            event_type,
            id: ev.id, // keep the id; you can ignore > 1 if you want single-touch only
        });
    }
}
