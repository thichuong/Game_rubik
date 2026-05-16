use bevy::prelude::*;

/// Event to reset the camera to its default orientation
#[derive(Message)]
pub struct ResetCameraEvent;

/// Event to trigger the step-by-step solver
#[derive(Message)]
pub struct SolveByStepsEvent;
