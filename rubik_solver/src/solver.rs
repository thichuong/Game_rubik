use crate::core::{CubieFace, FaceMapping};
use crate::helpers;
use bevy::prelude::*;
use kewb::{CubieCube, DataTable, FaceCube, Solver};

/// Pure function to invoke the Python solver for `NxN` (size >= 4) with a given state string.
/// This function does not reference any Bevy ECS structures, making it safe to run in async threads.
pub fn solve_nxn_state_only(state_str: &str) -> Option<Vec<String>> {
    let python_path = std::fs::canonicalize(".venv/bin/python")
        .unwrap_or_else(|_| std::path::PathBuf::from(".venv/bin/python"));
    let script_path =
        std::fs::canonicalize("python_solver/rubiks-cube-NxNxN-solver/rubiks-cube-solver.py")
            .unwrap_or_else(|_| std::path::PathBuf::from("rubiks-cube-solver.py"));

    // Obtain and inject the absolute path of .venv/bin to PATH env to let kociemba CLI execute correctly
    let mut path_env = std::env::var("PATH").unwrap_or_default();
    let venv_bin_absolute = std::fs::canonicalize(".venv/bin").ok().map_or_else(
        || ".venv/bin".to_string(),
        |p| p.to_string_lossy().into_owned(),
    );
    path_env = format!("{venv_bin_absolute}:{path_env}");

    let output = std::process::Command::new(python_path)
        .arg(script_path)
        .arg("--state")
        .arg(state_str)
        .current_dir("python_solver/rubiks-cube-NxNxN-solver")
        .env("PATH", path_env)
        .output()
        .ok()?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        let out_msg = String::from_utf8_lossy(&output.stdout);
        eprintln!("Python solver failed with status: {:?}", output.status);
        eprintln!("Stdout: {out_msg}");
        eprintln!("Stderr: {err_msg}");
        return None;
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);

    // Find the Solution: line and split logical moves
    for line in stdout_str.lines() {
        if line.starts_with("Solution: ") {
            let sol_part = line.trim_start_matches("Solution: ").trim();
            if sol_part.is_empty() {
                return Some(Vec::new());
            }
            let moves = sol_part
                .split_whitespace()
                .map(String::from)
                .collect::<Vec<String>>();
            return Some(moves);
        }
    }
    None
}

/// Unified solver function for all supported cube sizes.
/// It fetches the cube state using the relevant mapping and solves it using the Kociemba table or Python solver.
#[allow(clippy::cast_sign_loss)]
pub fn solve_cube_for_size(
    size: i32,
    faces: &Query<(&CubieFace, &GlobalTransform)>,
    cube_transform: &GlobalTransform,
    mapping: FaceMapping,
    table: &DataTable,
) -> Option<Vec<String>> {
    if size >= 4 {
        // Scrape the current logical cube state using Bevy entities
        let state =
            crate::nxn::state::NxNState::from_bevy(size as usize, faces, cube_transform, mapping)?;
        let state_str = state.to_string_rep();
        solve_nxn_state_only(&state_str)
    } else {
        let state_str = helpers::get_cube_state_for_size(size, faces, cube_transform, mapping)?;
        solve_cube(&state_str, table)
    }
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
