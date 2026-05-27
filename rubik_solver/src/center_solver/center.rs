// Main center solver implementation for nxn Rubik's cubes using orbit commutators.
// All comments in source files must be in English.

use crate::center_solver::commutator::find_any_solving_commutator;
use crate::center_solver::orbit::{Orbit, decompose_orbits};
use crate::cube::{Cube, CubeError, Face};

/// Solves all the mobile center pieces of the cube.
/// Returns the list of moves required to solve the centers.
pub fn solve_centers(cube: &mut Cube) -> Result<Vec<String>, CubeError> {
    let size = cube.size();
    if size < 4 {
        return Ok(Vec::new()); // No mobile centers to solve for size < 4
    }

    let mut final_moves = Vec::new();

    // --- PHASE 1: GLOBAL PARITY RESOLUTION via GF(2) ---
    // Mathematically resolve all odd parity states in the centers
    // using Gaussian elimination before executing any actual solving moves.
    let mut parity_moves = resolve_all_parities(cube)?;
    final_moves.append(&mut parity_moves);

    // --- PHASE 2: ACTUAL SOLVING ---
    // Now that all orbits are mathematically guaranteed to have EVEN parity,
    // solve them sequentially in absolute priority order without deadlock risks.
    let orbits = decompose_orbits(size);
    for orbit in &orbits {
        let mut orbit_moves = solve_single_orbit(cube, orbit)?;
        final_moves.append(&mut orbit_moves);
    }

    // Double check that all mobile centers are indeed solved
    for face_idx in 0..6 {
        let face = match face_idx {
            0 => Face::U,
            1 => Face::D,
            2 => Face::F,
            3 => Face::B,
            4 => Face::L,
            _ => Face::R,
        };
        for r in 1..(size - 1) {
            for c in 1..(size - 1) {
                if size % 2 == 1 && r == size / 2 && c == size / 2 {
                    continue; // Skip central fixed center
                }
                let val = cube.get(face, r, c)?;
                if val != face {
                    return Err(CubeError::InvalidMove(format!(
                        "Solver finished but center at {:?}({},{}) is unsolved: expected {:?}, got {:?}",
                        face, r, c, face, val
                    )));
                }
            }
        }
    }

    Ok(final_moves)
}

/// Computes the exact parity of a single center orbit using canonical cycle decomposition.
/// Returns true if the orbit has ODD parity, false if it has EVEN parity.
pub fn compute_orbit_parity(cube: &Cube, orbit: &Orbit) -> Result<bool, CubeError> {
    let len = orbit.pieces.len(); // Always 24
    let mut perm = vec![0; len];

    // Group the pieces by their target face.
    // For each face, we store the indices in `orbit.pieces` where `piece.face == face`.
    // There are always exactly 4 pieces per face.
    let faces = [Face::U, Face::D, Face::F, Face::B, Face::L, Face::R];

    for &face in &faces {
        // Find indices in `orbit.pieces` that belong to this face (destination positions)
        let mut dest_indices = Vec::with_capacity(4);
        for i in 0..len {
            if orbit.pieces[i].face == face {
                dest_indices.push(i);
            }
        }

        // Find indices in `orbit.pieces` that currently hold the color of this face
        let mut src_indices = Vec::with_capacity(4);
        for i in 0..len {
            let p = &orbit.pieces[i];
            let current_color = cube.get(p.face, p.row, p.col)?;
            if current_color == face {
                src_indices.push(i);
            }
        }

        // If a valid cube state, there must be exactly 4 pieces of each color in the orbit
        if dest_indices.len() != 4 || src_indices.len() != 4 {
            println!(
                "[PARITY ERROR] Orbit ({},{},{}) is invalid for face {:?}!",
                orbit.d_min, orbit.d_max, orbit.sub_orbit, face
            );
            for idx in 0..len {
                let p = &orbit.pieces[idx];
                let current_color = cube.get(p.face, p.row, p.col)?;
                println!(
                    "  Piece {} at {:?}({},{}) has color {:?}",
                    idx, p.face, p.row, p.col, current_color
                );
            }
            return Err(CubeError::InvalidMove(format!(
                "Invalid orbit state in ({},{},{}): found {} dest and {} src positions for face {:?}",
                orbit.d_min,
                orbit.d_max,
                orbit.sub_orbit,
                dest_indices.len(),
                src_indices.len(),
                face
            )));
        }

        // Match the source positions (holding color F) to destination positions (belonging to face F).
        // Sorting guarantees a canonical stable matching.
        for k in 0..4 {
            perm[src_indices[k]] = dest_indices[k];
        }
    }

    // Use the public helper from commutator.rs to determine parity
    Ok(crate::center_solver::commutator::is_odd_permutation(&perm))
}

/// Resolves all odd parity states in the centers using GF(2) linear algebra.
/// Finds the minimal set of slice moves to flip all orbits to even parity.
pub fn resolve_all_parities(cube: &mut Cube) -> Result<Vec<String>, CubeError> {
    let size = cube.size();
    let orbits = decompose_orbits(size);
    let num_orbits = orbits.len();

    // Define all independent slice moves (3 axes * (size - 2) layers)
    let mut candidate_moves = Vec::new();
    for layer in 1..(size - 1) {
        candidate_moves.push(format!("r{}", layer));
        candidate_moves.push(format!("u{}", layer));
        candidate_moves.push(format!("f{}", layer));
    }
    let num_moves = candidate_moves.len();

    // 1. Build the parity interaction matrix M[orbit_idx][move_idx]
    let mut matrix = vec![vec![0u8; num_moves]; num_orbits];
    for j in 0..num_moves {
        let mv = &candidate_moves[j];
        let mut solved_cube = Cube::new(size)?;
        solved_cube.apply_move(mv)?;
        for i in 0..num_orbits {
            if compute_orbit_parity(&solved_cube, &orbits[i])? {
                matrix[i][j] = 1;
            }
        }
    }

    // 2. Build the current parity vector P[orbit_idx]
    let mut p_vector = vec![0u8; num_orbits];
    let mut has_odd_parity = false;
    for i in 0..num_orbits {
        if compute_orbit_parity(cube, &orbits[i])? {
            p_vector[i] = 1;
            has_odd_parity = true;
        }
    }

    // If all orbits are already even, no slice moves are needed!
    if !has_odd_parity {
        return Ok(Vec::new());
    }

    // 3. Build augmented matrix A = [M | P]
    let mut a = vec![vec![0u8; num_moves + 1]; num_orbits];
    for i in 0..num_orbits {
        for j in 0..num_moves {
            a[i][j] = matrix[i][j];
        }
        a[i][num_moves] = p_vector[i];
    }

    // 4. Perform Gaussian Elimination on GF(2)
    let mut r = 0;
    let mut pivot_cols = vec![None; num_orbits];
    for c in 0..num_moves {
        if r >= num_orbits {
            break;
        }
        let mut pivot_row = None;
        for i in r..num_orbits {
            if a[i][c] == 1 {
                pivot_row = Some(i);
                break;
            }
        }

        if let Some(p_row) = pivot_row {
            a.swap(r, p_row);
            pivot_cols[r] = Some(c);

            for i in 0..num_orbits {
                if i != r && a[i][c] == 1 {
                    for col in c..=num_moves {
                        a[i][col] ^= a[r][col];
                    }
                }
            }
            r += 1;
        }
    }

    // 5. Check for unsolvable states (contradictions)
    for i in r..num_orbits {
        if a[i][num_moves] == 1 {
            return Err(CubeError::InvalidMove(
                "Odd center parity state is mathematically unsolvable on this cube size"
                    .to_string(),
            ));
        }
    }

    // 6. Extract a particular solution
    let mut x = vec![0u8; num_moves];
    for i in 0..r {
        if let Some(c) = pivot_cols[i] {
            x[c] = a[i][num_moves];
        }
    }

    // 7. Apply the required slice moves to the cube
    let mut parity_moves = Vec::new();
    for j in 0..num_moves {
        if x[j] == 1 {
            let mv = &candidate_moves[j];
            cube.apply_move(mv)?;
            parity_moves.push(mv.to_string());
        }
    }

    // Log the resolved parity moves
    if !parity_moves.is_empty() {
        println!(
            "[PARITY RESOLUTION] Odd parity detected. Resolved with: {:?}",
            parity_moves
        );
    }

    Ok(parity_moves)
}

/// Solves a single center orbit using 3-cycle commutators.
fn solve_single_orbit(cube: &mut Cube, orbit: &Orbit) -> Result<Vec<String>, CubeError> {
    let mut moves = Vec::new();
    let size = cube.size();
    let fallback_count = 0;

    let mut iterations = 0;
    // Set max_iterations to 120 to allow sufficient breakout face turns
    let max_iterations = 120;

    let mut min_unsolved = usize::MAX;
    let mut no_progress_count = 0;

    while iterations < max_iterations {
        iterations += 1;

        // 1. Identify all unsolved pieces in this orbit
        let mut unsolved_count = 0;
        for &piece in &orbit.pieces {
            let label = cube.get(piece.face, piece.row, piece.col)?;
            if label != piece.face {
                unsolved_count += 1;
            }
        }

        if unsolved_count == 0 {
            break; // Orbit is fully solved!
        }

        // Deadlock / Progress tracking
        if unsolved_count < min_unsolved {
            min_unsolved = unsolved_count;
            no_progress_count = 0;
        } else {
            no_progress_count += 1;
        }

        // Human-like Breakout: If we are stuck in a deadlock loop (no progress for 4 iterations),
        // we apply a random outer face move. This is mathematically guaranteed to preserve
        // the color state of all previously solved orbits, but changes the piece configuration
        // in the current orbit to break the deadlock.
        if no_progress_count >= 4 {
            let face_moves = [
                "U", "D", "F", "B", "L", "R", "U'", "D'", "F'", "B'", "L'", "R'",
            ];
            let breakout_move = face_moves[(iterations + fallback_count) % face_moves.len()];
            println!(
                "[BREAKOUT] Stuck in orbit ({},{},{}) with {} unsolved. Applying face move: {}",
                orbit.d_min, orbit.d_max, orbit.sub_orbit, unsolved_count, breakout_move
            );
            cube.apply_move(breakout_move)?;
            moves.push(breakout_move.to_string());
            no_progress_count = 0; // Reset progress count to give the solver a new chance
            continue; // Proceed to the next iteration to find commutator on the new state
        }

        if no_progress_count >= 20 {
            println!("\n[DEADLOCK DETECTED] Visualizing cube state before termination:");
            cube.print_net();
            println!();
            return Err(CubeError::InvalidMove(format!(
                "Deadlock in orbit ({},{},{}) at iteration {}: unsolved={}",
                orbit.d_min, orbit.d_max, orbit.sub_orbit, iterations, unsolved_count
            )));
        }

        // Build solved mask
        let len = orbit.pieces.len();
        let mut solved_mask = vec![false; len];
        for i in 0..len {
            let p = &orbit.pieces[i];
            solved_mask[i] = cube.get(p.face, p.row, p.col)? == p.face;
        }

        // Try to find ANY commutator that makes solving progress (dry_run = false, enforce global preservation)
        match find_any_solving_commutator(size, cube, orbit, &solved_mask, false) {
            Ok(comm_moves) => {
                for m in &comm_moves {
                    cube.apply_move(m)?;
                }
                moves.extend(comm_moves);
                no_progress_count = 0; // Reset progress count on successful commutator application
            }
            Err(err) => {
                println!(
                    "[BREAKOUT-FALLBACK] Stopped solver because no commutator could be found for orbit ({},{},{}) with {} unsolved.",
                    orbit.d_min, orbit.d_max, orbit.sub_orbit, unsolved_count
                );
                return Err(CubeError::InvalidMove(format!(
                    "Stopped at BREAKOUT-FALLBACK for orbit ({},{},{}): {:?}",
                    orbit.d_min, orbit.d_max, orbit.sub_orbit, err
                )));
            }
        }
    }

    // Verify that the orbit is indeed fully solved
    let mut unsolved_count = 0;
    for &piece in &orbit.pieces {
        let label = cube.get(piece.face, piece.row, piece.col)?;
        if label != piece.face {
            unsolved_count += 1;
        }
    }
    if unsolved_count > 0 {
        return Err(CubeError::InvalidMove(format!(
            "Failed to solve orbit ({},{},{}) within {} iterations: unsolved={}",
            orbit.d_min, orbit.d_max, orbit.sub_orbit, max_iterations, unsolved_count
        )));
    }

    Ok(moves)
}
