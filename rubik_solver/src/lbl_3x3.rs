use crate::state::{Color, Cube};
use kewb::{CubieCube, DataTable, FaceCube, Solver};
use std::sync::LazyLock;

static DATA_TABLE: LazyLock<DataTable> = LazyLock::new(DataTable::default);

pub fn solve_3x3(cube: &mut Cube) -> Vec<String> {
    let state_str = to_3x3_string(cube);
    let table = &*DATA_TABLE;

    if let Ok(face_cube) = FaceCube::try_from(state_str.as_str()) {
        if let Ok(cubie_cube) = CubieCube::try_from(&face_cube) {
            let mut solver = Solver::new(table, 23, None);
            if let Some(sol) = solver.solve(cubie_cube) {
                return sol.to_string().split_whitespace().map(String::from).collect();
            }
        }
    }

    Vec::new()
}

fn to_3x3_string(cube: &Cube) -> String {
    let mut s = String::new();
    // kewb expects order: U, R, F, D, L, B
    let faces = [Color::U, Color::R, Color::F, Color::D, Color::L, Color::B];
    let size = cube.size;

    for &f in &faces {
        // Top-left corner
        s.push(cube.get_color(f, 0, 0).to_char());
        // Top-edge (center of the 3x3 face)
        s.push(cube.get_color(f, 0, size / 2).to_char());
        // Top-right
        s.push(cube.get_color(f, 0, size - 1).to_char());

        // Mid-left
        s.push(cube.get_color(f, size / 2, 0).to_char());
        // Center
        s.push(cube.get_color(f, size / 2, size / 2).to_char());
        // Mid-right
        s.push(cube.get_color(f, size / 2, size - 1).to_char());

        // Bottom-left
        s.push(cube.get_color(f, size - 1, 0).to_char());
        // Bottom-edge
        s.push(cube.get_color(f, size - 1, size / 2).to_char());
        // Bottom-right
        s.push(cube.get_color(f, size - 1, size - 1).to_char());
    }
    s
}
