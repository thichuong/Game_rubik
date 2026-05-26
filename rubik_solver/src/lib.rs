#![allow(clippy::manual_strip)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::collapsible_if)]

// Declares modules and sets up the public APIs.
// All comments in source files must be in English.

pub mod center_solver;
pub mod core;
pub mod cube;
pub mod helpers;
pub mod plugin;
pub mod resources;
pub mod solver;

// Export Bevy game core types for the root workspace
pub use core::{CubieFace, Direction, Face, FaceMapping, RotationAxis, RotationMove};

// Export Bevy game integration resources and plugin
pub use plugin::SolverPlugin;
pub use resources::{SolverResource, StepByStepSolution};

// Export solver types
pub use cube::{Cube, CubeError};
