#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::doc_markdown,
    clippy::similar_names
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
    let mut final_3x3_state_str = String::new();
    let mut best_combo = None;

    let base_3x3_str = map_to_3x3_string(&state);

    let combo_attempts = [
        (false, false), // No parities
        (true, false),  // OLL only
        (false, true),  // PLL only
        (true, true),   // OLL + PLL
    ];

    for &(try_oll, try_pll) in &combo_attempts {
        let mut temp_3x3 = base_3x3_str.clone();
        if try_oll {
            temp_3x3 = crate::nxn::parity::apply_oll_parity_to_string(&temp_3x3);
        }
        if try_pll {
            temp_3x3 = crate::nxn::parity::apply_pll_parity_to_string(&temp_3x3);
        }

        if is_solvable_3x3(&temp_3x3) {
            best_combo = Some((try_oll, try_pll));
            final_3x3_state_str = temp_3x3;
            break;
        }
    }

    let (need_oll, need_pll) = best_combo?;
    let mut parity_moves = Vec::new();

    if need_oll {
        let oll_moves = get_oll_parity_moves(size);
        parity_moves.extend(oll_moves);
    }
    if need_pll {
        let pll_moves = get_pll_parity_moves(size);
        parity_moves.extend(pll_moves);
    }

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
