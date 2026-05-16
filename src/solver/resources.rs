use bevy::prelude::*;
use kewb::DataTable;

#[derive(Resource)]
pub struct SolverResource {
    pub table: DataTable,
}

#[derive(Resource, Default)]
pub struct StepByStepSolution {
    pub moves: Vec<String>,
    pub current_step: usize,
    pub active: bool,
}
