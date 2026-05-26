use crate::resources::{SolverResource, StepByStepSolution};
use bevy::prelude::*;
use std::sync::Arc;

pub struct SolverPlugin;

impl Plugin for SolverPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SolverResource {
            table: Arc::new(kewb::DataTable::default()),
        })
        .init_resource::<StepByStepSolution>();
    }
}
