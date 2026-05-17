use bevy::prelude::*;

/// Event to reset the camera to its default orientation
#[derive(Message)]
pub struct ResetCameraEvent;

/// Event to trigger the step-by-step solver
#[derive(Message)]
pub struct SolveByStepsEvent;

/// Event from UDP camera tracking to rotate the rubik cube
#[derive(Message)]
pub struct HandRotationEvent {
    pub delta_x: f32,
    pub delta_y: f32,
}

/// Event containing the camera frame for UI display
#[derive(Message)]
pub struct CameraFrameEvent {
    pub frame_rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}
