#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::doc_markdown
)]

use crate::core::RotationMove;
use crate::nxn::centers::solve_centers;
use crate::nxn::edges::pair_edges;
use crate::nxn::parity::{
    get_oll_parity_moves, get_pll_parity_moves, is_solvable_3x3, map_to_3x3_string,
};
use crate::nxn::state::NxNState;
use kewb::DataTable;

/// Unified solver logic for NxN cubes (N >= 4) using the Reduction method
pub fn solve_nxn(
    size: usize,
    faces: &bevy::prelude::Query<(&crate::core::CubieFace, &bevy::prelude::GlobalTransform)>,
    cube_transform: &bevy::prelude::GlobalTransform,
    mapping: crate::core::FaceMapping,
    table: &DataTable,
) -> Option<Vec<RotationMove>> {
    // 1. Initialize logic state from Bevy
    let mut state = NxNState::from_bevy(size, faces, cube_transform, mapping)?;
    let mut all_moves = Vec::new();

    // 2. Solve center pieces
    let center_moves = solve_centers(&mut state)?;
    all_moves.extend(center_moves);

    // 3. Pair edge wing pieces
    let edge_moves = pair_edges(&mut state)?;
    all_moves.extend(edge_moves);

    // 4. Parity Verification and Correction
    // Try the 4 parity correction combinations to find the solvable 3x3 state
    let mut solved_parity_moves = None;
    let mut final_3x3_state_str = String::new();

    let combo_attempts = vec![
        (false, false), // No parities
        (true, false),  // OLL only
        (false, true),  // PLL only
        (true, true),   // OLL + PLL
    ];

    for (try_oll, try_pll) in combo_attempts {
        let mut temp_state = state.clone();
        let mut temp_moves = Vec::new();

        if try_oll {
            let oll_moves = get_oll_parity_moves(size);
            temp_state.apply_moves(&oll_moves);
            temp_moves.extend(oll_moves);
        }

        if try_pll {
            let pll_moves = get_pll_parity_moves(size);
            temp_state.apply_moves(&pll_moves);
            temp_moves.extend(pll_moves);
        }

        let state_3x3 = map_to_3x3_string(&temp_state);
        if is_solvable_3x3(&state_3x3) {
            solved_parity_moves = Some(temp_moves);
            final_3x3_state_str = state_3x3;
            break;
        }
    }

    let parity_moves = solved_parity_moves?;
    state.apply_moves(&parity_moves);
    all_moves.extend(parity_moves);

    // 5. Solve the mapped 3x3x3 state using the Kociemba 2-phase library
    let face_cube = kewb::FaceCube::try_from(final_3x3_state_str.as_str()).ok()?;
    let cubie_cube = kewb::CubieCube::try_from(&face_cube).ok()?;
    let mut solver = kewb::Solver::new(table, 23, None);
    let sol = solver.solve(cubie_cube)?;

    // 6. Translate the Kociemba moves into physical rotations
    let sol_str = sol.to_string();
    let solution_moves = crate::helpers::solution_to_moves(&sol_str, size as i32, mapping);
    all_moves.extend(solution_moves);

    // Optimize the moves using merge reduction
    Some(crate::helpers::optimize_moves(&all_moves))
}
