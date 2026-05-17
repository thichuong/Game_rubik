use crate::rubik::resources::SkinType;
use bevy::prelude::*;

#[derive(Component)]
pub struct ShuffleButton;

#[derive(Component)]
pub struct SolveButton;

#[derive(Component)]
pub struct NextStepButton;

#[derive(Component)]
pub struct SolutionPanel;

#[derive(Component)]
pub struct StepText;

#[derive(Component)]
pub struct CloseButton;

#[derive(Component)]
pub struct SkinButton(pub SkinType);

#[derive(Component)]
pub struct SkinToggleButton;

#[derive(Component)]
pub struct SkinList;

#[derive(Component)]
pub struct EnvToggleButton;

#[derive(Component)]
pub struct EnvList;

#[derive(Component)]
pub enum EnvControl {
    Intensity(f32), // Increment/Decrement value
    Temp(Color),    // Presets for temperature
    Angle(f32),     // Increment/Decrement in radians
    Bg(Color),      // Preset backgrounds
}
