use crate::events::ResetCameraEvent;
use crate::rubik::components::RubikCube;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

/// System to handle whole-cube rotation via RMB (Free 360-degree rotation)
pub fn update_rubik_rotation(
    mouse_button: Res<ButtonInput<MouseButton>>,
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    mut hand_rotation_events: MessageReader<crate::events::HandRotationEvent>,
    cube_query: Single<&mut Transform, With<RubikCube>>,
    camera_query: Single<&Transform, (With<Camera>, Without<RubikCube>)>,
) {
    let mut delta_x = 0.0;
    let mut delta_y = 0.0;

    if mouse_button.pressed(MouseButton::Right) {
        delta_x += accumulated_mouse_motion.delta.x;
        delta_y += accumulated_mouse_motion.delta.y;
    }

    for event in hand_rotation_events.read() {
        delta_x += event.delta_x;
        delta_y += event.delta_y;
    }

    if delta_x != 0.0 || delta_y != 0.0 {
        let mut transform = cube_query.into_inner();
        let cam_transform = *camera_query;

        let dx = delta_x * 0.005;
        let dy = delta_y * 0.005;

        // Horizontal drag -> rotate around camera's up axis (screen vertical)
        let rot_y = Quat::from_axis_angle(*cam_transform.up(), dx);
        // Vertical drag -> rotate around camera's right axis (screen horizontal)
        let rot_x = Quat::from_axis_angle(*cam_transform.right(), dy);

        // Apply rotation in world space relative to camera perspective
        transform.rotation = rot_y * rot_x * transform.rotation;
    }
}

/// System to handle cube rotation reset
pub fn handle_cube_reset(
    mut events: MessageReader<ResetCameraEvent>,
    mut cube_transform: Single<&mut Transform, With<RubikCube>>,
    face_mapping: Res<crate::rubik::resources::FaceMapping>,
) {
    for _ in events.read() {
        cube_transform.rotation = face_mapping.get_rotation();
    }
}
