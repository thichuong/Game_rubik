<<<<<<< HEAD
#![allow(clippy::manual_strip)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::collapsible_if)]

// Declares modules and sets up the public APIs.
// All comments in source files must be in English.

pub mod cube;
pub mod solver;

pub use cube::{Cube, CubeError, Face};
=======
#![allow(clippy::must_use_candidate)]

pub mod camera;
pub mod environment;
pub mod events;
pub mod input;
pub mod rubik;
pub mod ui;
>>>>>>> 14bedf2fde708ad88f3e393c99ad64f65d77917e
