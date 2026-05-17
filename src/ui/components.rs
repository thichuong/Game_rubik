use crate::rubik::components::Face;
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

#[derive(Component)]
pub struct SizeSliderTrack;

#[derive(Component)]
pub struct SizeSliderHandle;

#[derive(Component)]
pub struct SizeSliderFill;

#[derive(Component)]
pub struct SizeDecrementButton;

#[derive(Component)]
pub struct SizeIncrementButton;

#[derive(Component)]
pub struct SizeText;

#[derive(Component)]
pub struct SolveButtonText;

// Face Mapping UI Components
#[derive(Component)]
pub struct MappingToggleButton;

#[derive(Component)]
pub struct MappingList;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingControl {
    ToggleOrder,
    SelectF(Face),
    SelectD(Face),
}

#[derive(Component)]
pub struct MappingOrderText;

#[derive(Component)]
pub struct SidebarScrollable;

#[derive(Component)]
pub struct ScrollContentWrapper;

#[derive(Component)]
pub struct SidebarScrollTrack;

#[derive(Component)]
pub struct SidebarScrollHandle;

#[derive(Resource, Default)]
pub struct SidebarScrollState {
    pub is_dragging: bool,
    pub drag_start_cursor_y: f32,
    pub drag_start_scroll_y: f32,
}
