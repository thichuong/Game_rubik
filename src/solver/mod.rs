pub mod helpers;
pub mod resources;

use bevy::prelude::*;
use resources::SolverResource;

pub struct SolverPlugin;

impl Plugin for SolverPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SolverResource {
            table: kewb::DataTable::default(),
        })
        .init_resource::<resources::StepByStepSolution>();
    }
}
