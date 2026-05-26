// Main center solver implementation for nxn Rubik's cubes using orbit commutators.
// All comments in source files must be in English.

use crate::cube::{Cube, CubeError, Face};
use crate::solver::commutator::find_any_solving_commutator;
use crate::solver::orbit::{Orbit, decompose_orbits};

/// Solves all the mobile center pieces of the cube.
/// Returns the list of moves required to solve the centers.
pub fn solve_centers(cube: &mut Cube) -> Result<Vec<String>, CubeError> {
    let size = cube.size();
    if size < 4 {
        return Ok(Vec::new()); // No mobile centers to solve for size < 4
    }

    let orbits = decompose_orbits(size);
    let mut all_moves = Vec::new();

    // --- PHASE 1: GLOBAL PARITY RESOLUTION (DRY RUN SCAN) ---
    // Mathematically analyze and resolve all odd parity states in the centers
    // before executing any actual solving moves.
    let mut scan_iterations = 0;
    let max_scan_iterations = 12;

    while scan_iterations < max_scan_iterations {
        scan_iterations += 1;

        let mut odd_orbits = Vec::new();
        for i in 0..orbits.len() {
            let orbit = &orbits[i];
            let mut virtual_cube = cube.clone();
            if dry_run_solve_orbit(&mut virtual_cube, orbit).is_err() {
                odd_orbits.push(i);
            }
        }

        if odd_orbits.is_empty() {
            break; // All orbits are mathematically guaranteed to have EVEN parity!
        }

        // Fix parity of the first odd orbit by applying a 90-degree slice turn
        let target_orbit_idx = odd_orbits[0];
        let target_orbit = &orbits[target_orbit_idx];
        let parity_breaker = get_parity_breaker_move(size, target_orbit, 0);

        // Ensure it is a 90-degree slice turn (not a double turn '2') to correctly flip parity.
        let clean_breaker = if parity_breaker.ends_with('2')
            && parity_breaker.len() >= 3
            && parity_breaker
                .chars()
                .nth(parity_breaker.len() - 2)
                .unwrap_or(' ')
                .is_ascii_digit()
        {
            parity_breaker[..parity_breaker.len() - 1].to_string()
        } else {
            parity_breaker
        };

        println!(
            "[PARITY RESOLUTION] Orbit ({},{},{}) detected as ODD. Flipping parity with: {}",
            target_orbit.d_min, target_orbit.d_max, target_orbit.sub_orbit, clean_breaker
        );
        cube.apply_move(&clean_breaker)?;
        all_moves.push(clean_breaker);
    }

    // --- PHASE 2: ACTUAL SOLVING ---
    // Now that all orbits are mathematically guaranteed to have EVEN parity,
    // solve them sequentially in absolute priority order without deadlock risks.
    for orbit in &orbits {
        let mut orbit_moves = solve_single_orbit(cube, orbit)?;
        all_moves.append(&mut orbit_moves);
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

    Ok(all_moves)
}

/// Helper to check if an orbit has even or odd parity by running a virtual solve.
fn dry_run_solve_orbit(cube: &mut Cube, orbit: &Orbit) -> Result<(), CubeError> {
    let size = cube.size();
    let mut iterations = 0;
    let max_iterations = 50;

    while iterations < max_iterations {
        iterations += 1;

        let mut unsolved_count = 0;
        for &p in &orbit.pieces {
            if cube.get(p.face, p.row, p.col)? != p.face {
                unsolved_count += 1;
            }
        }

        if unsolved_count == 0 {
            return Ok(()); // Solved! Even parity confirmed.
        }

        let len = orbit.pieces.len();
        let mut solved_mask = vec![false; len];
        for i in 0..len {
            let p = &orbit.pieces[i];
            solved_mask[i] = cube.get(p.face, p.row, p.col)? == p.face;
        }

        // Dry run is allowed to solve without global preservation to purely verify the orbit's internal parity
        match find_any_solving_commutator(size, cube, orbit, &solved_mask, true) {
            Ok(comm_moves) => {
                for m in &comm_moves {
                    cube.apply_move(m)?;
                }
            }
            Err(_) => {
                return Err(CubeError::InvalidMove("Odd parity detected".to_string()));
            }
        }
    }

    Err(CubeError::InvalidMove("Max iterations reached".to_string()))
}

/// Solves a single center orbit using 3-cycle commutators.
fn solve_single_orbit(cube: &mut Cube, orbit: &Orbit) -> Result<Vec<String>, CubeError> {
    let mut moves = Vec::new();
    let size = cube.size();
    let mut fallback_count = 0;

    let mut iterations = 0;
    let max_iterations = 80;

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

        if no_progress_count >= 12 {
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
            }
            Err(_) => {
                // Parity safety net (should never happen because Phase 1 already guaranteed Even parity)
                let fallback_move = get_parity_breaker_move(size, orbit, fallback_count);
                fallback_count += 1;

                let clean_fallback = if fallback_move.ends_with('2')
                    && fallback_move.len() >= 3
                    && fallback_move
                        .chars()
                        .nth(fallback_move.len() - 2)
                        .unwrap_or(' ')
                        .is_ascii_digit()
                {
                    fallback_move[..fallback_move.len() - 1].to_string()
                } else {
                    fallback_move
                };

                println!(
                    "[DEBUG] Parity safety net triggered. Applying fallback: {}",
                    clean_fallback
                );
                cube.apply_move(&clean_fallback)?;
                moves.push(clean_fallback);
            }
        }
    }

    Ok(moves)
}

/// Generates a single slice move to break parity or resolve deadlocks in an orbit.
fn get_parity_breaker_move(size: usize, orbit: &Orbit, count: usize) -> String {
    // We build a list of all physical slice indices that interact with this orbit.
    let mut interactive_slices = vec![
        orbit.d_min,
        orbit.d_max,
        size - 1 - orbit.d_min,
        size - 1 - orbit.d_max,
    ];

    interactive_slices.sort();
    interactive_slices.dedup();

    let valid_slices: Vec<usize> = interactive_slices
        .into_iter()
        .filter(|&idx| idx > 0 && idx < size - 1)
        .collect();

    let slice_idx = if valid_slices.is_empty() {
        size - 1 - orbit.d_min
    } else {
        valid_slices[count % valid_slices.len()]
    };

    let slice_types = ['r', 'u', 'f'];
    let modifiers = ["", "'", "2"];

    let num_slices = if valid_slices.is_empty() {
        1
    } else {
        valid_slices.len()
    };
    let s_type = slice_types[(count / (num_slices * 3)) % 3];
    let modifier = modifiers[(count / num_slices) % 3];

    format!("{}{}{}", s_type, slice_idx, modifier)
}
