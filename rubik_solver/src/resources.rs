use bevy::prelude::*;
use kewb::DataTable;
use std::sync::Arc;

#[derive(Resource)]
pub struct SolverResource {
    pub table: Arc<DataTable>,
}

#[derive(Resource, Default)]
pub struct StepByStepSolution {
    pub moves: Vec<String>,
    pub current_step: usize,
    pub active: bool,
    pub failed: bool,
    // Indicates if the solver is currently computing a solution in the background
    pub is_searching: bool,
}
