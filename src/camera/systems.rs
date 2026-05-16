use crate::camera::components::OrbitCamera;
use crate::events::ResetCameraEvent;
use bevy::prelude::*;

/// System to handle camera orbiting via mouse input
pub fn update_camera_orbit(orbit_query: Single<(&mut Transform, &mut OrbitCamera)>) {
    let (mut transform, mut orbit) = orbit_query.into_inner();

    // Removed Right Mouse Button handling here - moved to RubikCube rotation

    orbit.beta = orbit.beta.clamp(-1.4, 1.4); // Limit pitch

    let x = orbit.radius * orbit.beta.cos() * orbit.alpha.sin();
    let y = orbit.radius * orbit.beta.sin();
    let z = orbit.radius * orbit.beta.cos() * orbit.alpha.cos();

    transform.translation = Vec3::new(x, y, z);
    transform.look_at(Vec3::ZERO, Vec3::Y);
}

/// System to handle camera reset event
pub fn handle_camera_reset(
    mut events: MessageReader<ResetCameraEvent>,
    mut orbit: Single<&mut OrbitCamera>,
) {
    for _ in events.read() {
        orbit.alpha = 0.785;
        orbit.beta = 0.785;
    }
}
