use crate::events::ResetCameraEvent;
use crate::rubik::components::RubikCube;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

/// System to handle whole-cube rotation via RMB (Free 360-degree rotation)
pub fn update_rubik_rotation(
    mouse_button: Res<ButtonInput<MouseButton>>,
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    cube_query: Single<&mut Transform, With<RubikCube>>,
    camera_query: Single<&Transform, (With<Camera>, Without<RubikCube>)>,
) {
    if mouse_button.pressed(MouseButton::Right) {
        let mut transform = cube_query.into_inner();
        let cam_transform = *camera_query;

        let delta_x = accumulated_mouse_motion.delta.x * 0.005;
        let delta_y = accumulated_mouse_motion.delta.y * 0.005;

        // Horizontal drag -> rotate around camera's up axis (screen vertical)
        let rot_y = Quat::from_axis_angle(*cam_transform.up(), delta_x);
        // Vertical drag -> rotate around camera's right axis (screen horizontal)
        let rot_x = Quat::from_axis_angle(*cam_transform.right(), delta_y);

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
