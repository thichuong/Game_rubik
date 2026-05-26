use bevy::prelude::*;

/// Component for the orbiting camera
#[derive(Component)]
pub struct OrbitCamera {
    pub radius: f32,
    pub alpha: f32, // Horizontal angle (yaw)
    pub beta: f32,  // Vertical angle (pitch)
}
