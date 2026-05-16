use crate::rubik::components::{Direction, RotationAxis, RotationMove};
use bevy::prelude::*;
use std::collections::VecDeque;

/// Resource to hold materials for the Rubik's cube
#[derive(Resource)]
pub struct RubikMaterials {
    pub white: Handle<StandardMaterial>,
    pub yellow: Handle<StandardMaterial>,
    pub red: Handle<StandardMaterial>,
    pub orange: Handle<StandardMaterial>,
    pub green: Handle<StandardMaterial>,
    pub blue: Handle<StandardMaterial>,
    pub black: Handle<StandardMaterial>,
    // Skin textures
    pub carbon_tex: Handle<Image>,
    pub geometric_tex: Handle<Image>,
    pub floral_tex: Handle<Image>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SkinType {
    #[default]
    Classic,
    Carbon,
    Geometric,
    Floral,
}

#[derive(Resource, Default)]
pub struct RubikSkin {
    pub current: SkinType,
}

#[derive(Resource, Default)]
pub struct RotationQueue(pub VecDeque<RotationMove>);

#[derive(Resource, Default)]
pub struct MoveHistory {
    pub done: Vec<RotationMove>,
    pub undone: Vec<RotationMove>,
}

#[derive(Resource)]
pub struct CurrentlyRotating {
    pub axis: RotationAxis,
    pub index: i32,
    pub direction: Direction,
    pub timer: Timer,
    pub rotation_axis: Vec3, // Actual vector for Quat
    pub angle: f32,
    pub cubies: Vec<Entity>,
    pub add_to_history: bool,
}
