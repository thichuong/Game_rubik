use kewb::{CubieCube, DataTable, FaceCube, Solver};

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
