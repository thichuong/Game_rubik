use crate::core::{CubieFace, FaceMapping};
use crate::helpers;
use bevy::prelude::*;
use kewb::{CubieCube, DataTable, FaceCube, Solver};

/// Unified solver function for all supported cube sizes (2x2x2 and 3x3x3).
/// It fetches the cube state using the relevant mapping and solves it using the Kociemba table.
pub fn solve_cube_for_size(
    size: i32,
    faces: &Query<(&CubieFace, &GlobalTransform)>,
    cube_transform: &GlobalTransform,
    mapping: FaceMapping,
    table: &DataTable,
) -> Option<Vec<String>> {
    let state_str = helpers::get_cube_state_for_size(size, faces, cube_transform, mapping)?;
    solve_cube(&state_str, table)
}

pub fn solve_cube(state_str: &str, table: &DataTable) -> Option<Vec<String>> {
    let face_cube = FaceCube::try_from(state_str).ok()?;
    let cubie_cube = CubieCube::try_from(&face_cube).ok()?;
    let mut solver = Solver::new(table, 23, None);
    let sol = solver.solve(cubie_cube)?;
    Some(
        sol.to_string()
            .split_whitespace()
            .map(String::from)
            .collect(),
    )
}
