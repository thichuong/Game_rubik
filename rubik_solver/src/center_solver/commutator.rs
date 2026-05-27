// Dynamic generator and searcher for 3-cycle commutators using exact index tracking and color-based preservation.
// All comments in source files must be in English.

use crate::center_solver::orbit::{CenterPiece, Orbit, decompose_orbits};
use crate::cube::moves::{Axis, from_3d_to_rc, to_3d};
use crate::cube::{Cube, CubeError, Face};

/// Generates the inverse of a move string.
pub fn get_inverse_move(m: &str) -> String {
    if m.ends_with('\'') {
        m[..m.len() - 1].to_string()
    } else if m.ends_with('2') {
        m.to_string() // 180 degree turn is its own inverse
    } else {
        format!("{}'", m)
    }
}

/// Helper to generate candidate moves for the search space based on cube size and orbit coordinates.
fn get_candidate_moves(size: usize, orbit: &Orbit) -> (Vec<String>, Vec<String>) {
    let mut slice_moves = Vec::new();
    let mut face_moves = Vec::new();

    // Standard outer face moves
    let faces = ["U", "D", "F", "B", "L", "R"];
    for &f in &faces {
        face_moves.push(f.to_string());
        face_moves.push(format!("{}'", f));
        face_moves.push(format!("{}2", f));
    }

    // Slice moves related to this orbit
    let mut interactive_slices = std::collections::HashSet::new();
    interactive_slices.insert(orbit.d_min);
    interactive_slices.insert(orbit.d_max);
    interactive_slices.insert(size - 1 - orbit.d_min);
    interactive_slices.insert(size - 1 - orbit.d_max);

    let slice_types = ['u', 'd', 'r', 'l', 'f', 'b'];
    for &s in &slice_types {
        for &slice_idx in &interactive_slices {
            if slice_idx > 0 && slice_idx < size - 1 {
                slice_moves.push(format!("{}{}", s, slice_idx));
                slice_moves.push(format!("{}{}'", s, slice_idx));
                slice_moves.push(format!("{}{}2", s, slice_idx));
            }
        }
    }

    (slice_moves, face_moves)
}

/// Helper to simulate slice rotation on a flat index tracker vector.
fn rotate_slice_indices(
    size: usize,
    grid: &mut Vec<usize>,
    axis: Axis,
    slice_idx: usize,
    clockwise: bool,
) -> Result<(), CubeError> {
    if slice_idx >= size {
        return Err(CubeError::IndexOutOfBounds(slice_idx, size));
    }
    let n = size;
    let mut new_grid = grid.clone();

    for face_idx in 0..6 {
        let face = match face_idx {
            0 => Face::U,
            1 => Face::D,
            2 => Face::F,
            3 => Face::B,
            4 => Face::L,
            _ => Face::R,
        };

        for r in 0..n {
            for c in 0..n {
                let (x, y, z) = to_3d(face, r, c, n);

                let in_slice = match axis {
                    Axis::X => x == slice_idx,
                    Axis::Y => y == slice_idx,
                    Axis::Z => z == slice_idx,
                };

                if in_slice {
                    let (x_old, y_old, z_old) = match (axis, clockwise) {
                        (Axis::Y, true) => (z, y, n - 1 - x),
                        (Axis::Y, false) => (n - 1 - z, y, x),
                        (Axis::X, true) => (x, z, n - 1 - y),
                        (Axis::X, false) => (x, n - 1 - z, y),
                        (Axis::Z, true) => (n - 1 - y, x, z),
                        (Axis::Z, false) => (y, n - 1 - x, z),
                    };

                    let face_old = match (axis, clockwise) {
                        (Axis::Y, true) => match face {
                            Face::U => Face::U,
                            Face::D => Face::D,
                            Face::F => Face::R,
                            Face::R => Face::B,
                            Face::B => Face::L,
                            Face::L => Face::F,
                        },
                        (Axis::Y, false) => match face {
                            Face::U => Face::U,
                            Face::D => Face::D,
                            Face::F => Face::L,
                            Face::L => Face::B,
                            Face::B => Face::R,
                            Face::R => Face::F,
                        },
                        (Axis::X, true) => match face {
                            Face::R => Face::R,
                            Face::L => Face::L,
                            Face::U => Face::F,
                            Face::F => Face::D,
                            Face::D => Face::B,
                            Face::B => Face::U,
                        },
                        (Axis::X, false) => match face {
                            Face::R => Face::R,
                            Face::L => Face::L,
                            Face::U => Face::B,
                            Face::B => Face::D,
                            Face::D => Face::F,
                            Face::F => Face::U,
                        },
                        (Axis::Z, true) => match face {
                            Face::F => Face::F,
                            Face::B => Face::B,
                            Face::U => Face::L,
                            Face::L => Face::D,
                            Face::D => Face::R,
                            Face::R => Face::U,
                        },
                        (Axis::Z, false) => match face {
                            Face::F => Face::F,
                            Face::B => Face::B,
                            Face::U => Face::R,
                            Face::R => Face::D,
                            Face::D => Face::L,
                            Face::L => Face::U,
                        },
                    };

                    let (r_old, c_old) = from_3d_to_rc(face_old, x_old, y_old, z_old, n);
                    let source_idx = (face_old as usize) * n * n + r_old * n + c_old;
                    let source_val = grid[source_idx];

                    let idx = (face as usize) * n * n + r * n + c;
                    new_grid[idx] = source_val;
                }
            }
        }
    }

    *grid = new_grid;
    Ok(())
}

/// Helper to apply a move on a flat index tracker vector.
fn apply_move_to_indices(
    size: usize,
    grid: &mut Vec<usize>,
    move_str: &str,
) -> Result<(), CubeError> {
    if move_str.is_empty() {
        return Err(CubeError::InvalidMove(move_str.to_string()));
    }

    let mut base_str = move_str;
    let mut double = false;
    let mut clockwise = true;

    if move_str.ends_with('\'') {
        clockwise = false;
        base_str = &move_str[..move_str.len() - 1];
    } else if move_str.ends_with('2') {
        let first = move_str.chars().next().unwrap_or(' ');
        let is_outer = first.is_uppercase();
        let is_slice_double = first.is_lowercase()
            && move_str.len() >= 3
            && move_str
                .chars()
                .nth(move_str.len() - 2)
                .unwrap_or(' ')
                .is_ascii_digit();

        if is_outer || is_slice_double {
            double = true;
            base_str = &move_str[..move_str.len() - 1];
        }
    }

    let mut chars = base_str.chars();
    let first_char = chars
        .next()
        .ok_or_else(|| CubeError::InvalidMove(move_str.to_string()))?;

    let mut slice_num: Option<usize> = None;
    let move_type = first_char;

    if first_char.is_lowercase() {
        let mut num_str = String::new();
        for c in chars {
            if c.is_ascii_digit() {
                num_str.push(c);
            } else {
                return Err(CubeError::InvalidMove(move_str.to_string()));
            }
        }
        let val = num_str
            .parse::<usize>()
            .map_err(|_| CubeError::InvalidMove(move_str.to_string()))?;
        if val == 0 || val >= size - 1 {
            return Err(CubeError::InvalidMove(move_str.to_string()));
        }
        slice_num = Some(val);
    }

    let n = size;
    let (axis, slice_idx, axis_cw) = match move_type {
        'U' => (Axis::Y, n - 1, clockwise),
        'D' => (Axis::Y, 0, !clockwise),
        'R' => (Axis::X, n - 1, clockwise),
        'L' => (Axis::X, 0, !clockwise),
        'F' => (Axis::Z, n - 1, clockwise),
        'B' => (Axis::Z, 0, !clockwise),

        'u' => {
            let idx = slice_num.ok_or_else(|| CubeError::InvalidMove(move_str.to_string()))?;
            (Axis::Y, n - 1 - idx, clockwise)
        }
        'd' => {
            let idx = slice_num.ok_or_else(|| CubeError::InvalidMove(move_str.to_string()))?;
            (Axis::Y, idx, !clockwise)
        }
        'r' => {
            let idx = slice_num.ok_or_else(|| CubeError::InvalidMove(move_str.to_string()))?;
            (Axis::X, n - 1 - idx, clockwise)
        }
        'l' => {
            let idx = slice_num.ok_or_else(|| CubeError::InvalidMove(move_str.to_string()))?;
            (Axis::X, idx, !clockwise)
        }
        'f' => {
            let idx = slice_num.ok_or_else(|| CubeError::InvalidMove(move_str.to_string()))?;
            (Axis::Z, n - 1 - idx, clockwise)
        }
        'b' => {
            let idx = slice_num.ok_or_else(|| CubeError::InvalidMove(move_str.to_string()))?;
            (Axis::Z, idx, !clockwise)
        }
        _ => return Err(CubeError::InvalidMove(move_str.to_string())),
    };

    if double {
        rotate_slice_indices(size, grid, axis, slice_idx, axis_cw)?;
        rotate_slice_indices(size, grid, axis, slice_idx, axis_cw)?;
    } else {
        rotate_slice_indices(size, grid, axis, slice_idx, axis_cw)?;
    }

    Ok(())
}

/// Verifies a 3-cycle commutator sequence S + [X, Y] + S' to solve: A -> B -> C -> A.
/// Uses exact physical index tracking, yielding 100% correct verification in a single pass.
/// Enforces preservation of already solved pieces using color/face matching.
fn verify_3cycle(
    size: usize,
    cube: &Cube,
    orbit: &Orbit,
    seq: &[String],
    idx_a: usize,
    idx_b: usize,
    idx_c: usize,
    solved_mask: &[bool],
) -> Result<bool, CubeError> {
    // --- Step 1: Verify the 3-cycle itself using Index Tracking (Bijection guarantee) ---
    let mut grid: Vec<usize> = (0..(6 * size * size)).collect();
    for m in seq {
        apply_move_to_indices(size, &mut grid, m)?;
    }

    let len = orbit.pieces.len();
    let mut orbit_flat_indices = Vec::with_capacity(len);
    for p in &orbit.pieces {
        let flat_idx = (p.face as usize) * size * size + p.row * size + p.col;
        orbit_flat_indices.push(flat_idx);
    }

    let flat_a = orbit_flat_indices[idx_a];
    let flat_b = orbit_flat_indices[idx_b];
    let flat_c = orbit_flat_indices[idx_c];

    if grid[flat_b] != flat_a || grid[flat_c] != flat_b || grid[flat_a] != flat_c {
        return Ok(false);
    }

    // --- Step 2: Verify Color/Face Preservation for Solved Pieces ---
    let mut test_cube = cube.clone();
    for m in seq {
        test_cube.apply_move(m)?;
    }

    for i in 0..len {
        if i != idx_a && i != idx_b && i != idx_c && solved_mask[i] {
            let p_i = &orbit.pieces[i];
            let label = test_cube.get(p_i.face, p_i.row, p_i.col)?;
            if label != p_i.face {
                return Ok(false); // Solved piece was corrupted
            }
        }
    }

    Ok(true)
}

/// Finds a 3-cycle commutator sequence S + [X, Y] + S' to solve: A -> B -> C -> A.
/// - `size`: size of the cube.
/// - `cube`: current virtual cube state.
/// - `orbit`: the active orbit.
/// - `piece_a`, `piece_b`, `piece_c`: the 3 pieces to cycle.
/// - `solved_mask`: boolean mask indicating which of the 24 pieces in the orbit are already solved.
pub fn find_3cycle_commutator(
    size: usize,
    cube: &Cube,
    orbit: &Orbit,
    piece_a: &CenterPiece,
    piece_b: &CenterPiece,
    piece_c: &CenterPiece,
    solved_mask: &[bool],
) -> Result<Vec<String>, CubeError> {
    let idx_a = orbit
        .pieces
        .iter()
        .position(|p| p == piece_a)
        .ok_or_else(|| CubeError::InvalidMove("Piece A not in orbit".to_string()))?;
    let idx_b = orbit
        .pieces
        .iter()
        .position(|p| p == piece_b)
        .ok_or_else(|| CubeError::InvalidMove("Piece B not in orbit".to_string()))?;
    let idx_c = orbit
        .pieces
        .iter()
        .position(|p| p == piece_c)
        .ok_or_else(|| CubeError::InvalidMove("Piece C not in orbit".to_string()))?;

    let (slice_moves, face_moves) = get_candidate_moves(size, orbit);

    let mut y_candidates = Vec::new();
    y_candidates.extend(face_moves.clone());
    y_candidates.extend(slice_moves.clone());

    // Strategic prioritization: try Face setups first for maximum orbit isolation, then extend to Slice setups.
    let mut all_setup_candidates = face_moves.clone();
    all_setup_candidates.extend(slice_moves.clone());

    // 1. Try zero setup (pure 4-move commutator [X, Y])
    for x in &slice_moves {
        for y in &y_candidates {
            if x == y || x == &get_inverse_move(y) {
                continue;
            }
            let x_inv = get_inverse_move(x);
            let y_inv = get_inverse_move(y);

            let seq = vec![x.clone(), y.clone(), x_inv, y_inv];
            if verify_3cycle(size, cube, orbit, &seq, idx_a, idx_b, idx_c, solved_mask)? {
                return Ok(seq);
            }
        }
    }

    // 2. Try 1-move setup (6-move sequence: S + [X, Y] + S')
    for s in &all_setup_candidates {
        let s_inv = get_inverse_move(s);
        for x in &slice_moves {
            for y in &y_candidates {
                if x == y || x == &get_inverse_move(y) {
                    continue;
                }
                let x_inv = get_inverse_move(x);
                let y_inv = get_inverse_move(y);

                let seq = vec![s.clone(), x.clone(), y.clone(), x_inv, y_inv, s_inv.clone()];
                if verify_3cycle(size, cube, orbit, &seq, idx_a, idx_b, idx_c, solved_mask)? {
                    return Ok(seq);
                }
            }
        }
    }

    Err(CubeError::InvalidMove(format!(
        "Could not find commutator for 3-cycle: {:?} -> {:?} -> {:?}",
        piece_a, piece_b, piece_c
    )))
}

/// Finds ANY commutator sequence S + [X, Y] + S' that makes progress in solving the orbit
/// without corrupting already solved pieces.
/// Returns the sequence of moves.
/// Helper to calculate if a permutation is odd.
pub fn is_odd_permutation(perm: &[usize]) -> bool {
    let n = perm.len();
    let mut visited = vec![false; n];
    let mut transpositions = 0;

    for i in 0..n {
        if !visited[i] {
            let mut cycle_len = 0;
            let mut curr = i;
            while !visited[curr] {
                visited[curr] = true;
                curr = perm[curr];
                cycle_len += 1;
            }
            if cycle_len > 1 {
                transpositions += cycle_len - 1;
            }
        }
    }
    transpositions % 2 == 1
}

/// Finds ANY commutator sequence S + [X, Y] + S' that makes progress in solving the orbit
/// without corrupting already solved pieces.
/// Returns the sequence of moves.
pub fn find_any_solving_commutator(
    size: usize,
    cube: &Cube,
    orbit: &Orbit,
    solved_mask: &[bool],
    dry_run: bool,
) -> Result<Vec<String>, CubeError> {
    let (slice_moves, face_moves) = get_candidate_moves(size, orbit);

    let mut y_candidates = Vec::new();
    y_candidates.extend(face_moves.clone());
    y_candidates.extend(slice_moves.clone());

    // Strategic prioritization: try Face setups first for maximum orbit isolation, then extend to Slice setups.
    let mut all_setup_candidates = face_moves.clone();
    all_setup_candidates.extend(slice_moves.clone());
    let face_setup_candidates = face_moves.clone();

    let len = orbit.pieces.len();

    // Pre-calculate flat indices of all pieces in the orbit
    let mut orbit_flat_indices = Vec::with_capacity(len);
    for p in &orbit.pieces {
        let flat_idx = (p.face as usize) * size * size + p.row * size + p.col;
        orbit_flat_indices.push(flat_idx);
    }

    // Get current colors of all pieces in the orbit
    let mut current_colors = vec![Face::U; len];
    for i in 0..len {
        let p = &orbit.pieces[i];
        current_colors[i] = cube.get(p.face, p.row, p.col)?;
    }

    // Helper to evaluate a candidate sequence
    let evaluate_seq = |seq: &[String]| -> Result<usize, CubeError> {
        // 1. Simulate on flat index tracker grid
        let mut grid: Vec<usize> = (0..(6 * size * size)).collect();
        for m in seq {
            apply_move_to_indices(size, &mut grid, m)?;
        }

        // 2. Determine new color for each of the pieces in the orbit after rotation
        let mut new_colors = vec![Face::U; len];
        for i in 0..len {
            let flat_idx = orbit_flat_indices[i];
            let source_flat_idx = grid[flat_idx];

            let source_orbit_idx = orbit_flat_indices
                .iter()
                .position(|&x| x == source_flat_idx);
            if let Some(src_idx) = source_orbit_idx {
                new_colors[i] = current_colors[src_idx];
            } else {
                return Ok(0); // Invalid move affecting other orbits
            }
        }

        // 3. Calculate became_solved and became_unsolved
        let mut became_solved = 0;
        let mut became_unsolved = 0;
        let mut color_unchanged = true;
        for i in 0..len {
            let target_face = orbit.pieces[i].face;
            let old_solved = solved_mask[i];
            let new_solved = new_colors[i] == target_face;

            if !old_solved && new_solved {
                became_solved += 1;
            } else if old_solved && !new_solved {
                became_unsolved += 1;
            }
            if new_colors[i] != current_colors[i] {
                color_unchanged = false;
            }
        }

        // 4. Calculate permutation matrix on orbit to track physical parity flips
        let mut perm = vec![0; len];
        for i in 0..len {
            let flat_idx = orbit_flat_indices[i];
            let source_flat_idx = grid[flat_idx];
            if let Some(src_idx) = orbit_flat_indices
                .iter()
                .position(|&x| x == source_flat_idx)
            {
                perm[i] = src_idx;
            } else {
                return Ok(0);
            }
        }

        let is_odd = is_odd_permutation(&perm);

        // 5. Score the sequence: strictly prioritize solving steps, but accept color-preserving parity flips when stuck
        let score = if became_solved >= 1 && became_unsolved == 0 {
            3 // Perfect solving step (highest priority)
        } else if became_solved > became_unsolved {
            2 // Net positive progress step (allow temporary swaps)
        } else if became_solved == 0
            && became_unsolved == 0
            && color_unchanged
            && is_odd
            && !dry_run
        {
            1 // Parity flip step (color-preserving odd permutation to break parity without scrambling anything!)
        } else {
            0
        };

        if score > 0 {
            // Verify on virtual cube for safety
            let mut test_cube = cube.clone();
            for m in seq {
                test_cube.apply_move(m)?;
            }
            // Double check that all solved pieces in the current orbit are strictly preserved if score is 3
            if score == 3 {
                for i in 0..len {
                    if solved_mask[i] {
                        let p = &orbit.pieces[i];
                        if test_cube.get(p.face, p.row, p.col)? != p.face {
                            return Ok(0);
                        }
                    }
                }
            }

            // --- GLOBAL ORBIT PRESERVATION FILTER ---
            if !dry_run {
                // Only protect orbits that were already solved in the sequential pipeline (i.e. those before the current orbit).
                let all_orbits = decompose_orbits(size);
                for o in &all_orbits {
                    if o.d_min == orbit.d_min
                        && o.d_max == orbit.d_max
                        && o.sub_orbit == orbit.sub_orbit
                    {
                        break;
                    }
                    for p in &o.pieces {
                        let is_solved_before = cube.get(p.face, p.row, p.col)? == p.face;
                        if is_solved_before && test_cube.get(p.face, p.row, p.col)? != p.face {
                            return Ok(0); // Reject to protect previously solved orbits
                        }
                    }
                }
            }
            // ----------------------------------------

            return Ok(score);
        }

        Ok(0)
    };

    let mut best_seq = Vec::new();
    let mut best_score = 0;

    // 1. Try zero setup (pure 4-move commutator)
    // 1a. Highly optimized: [Slice, Face] and [Face, Slice] commutator (super clean, super fast, mathematically preferred)
    for x in &slice_moves {
        for y in &face_moves {
            if x == y || x == &get_inverse_move(y) {
                continue;
            }
            let x_inv = get_inverse_move(x);
            let y_inv = get_inverse_move(y);

            // Form 1: [Slice, Face] = X Y X' Y'
            let seq1 = vec![x.clone(), y.clone(), x_inv.clone(), y_inv.clone()];
            let score1 = evaluate_seq(&seq1)?;
            if score1 == 3 {
                return Ok(seq1); // Immediate return for perfect move
            } else if score1 > best_score {
                best_score = score1;
                best_seq = seq1;
            }

            // Form 2: [Face, Slice] = Y X Y' X'
            let seq2 = vec![y.clone(), x.clone(), y_inv.clone(), x_inv.clone()];
            let score2 = evaluate_seq(&seq2)?;
            if score2 == 3 {
                return Ok(seq2); // Immediate return for perfect move
            } else if score2 > best_score {
                best_score = score2;
                best_seq = seq2;
            }
        }
    }

    // 1b. Fallback: [Slice, Slice] commutator
    if best_score < 3 {
        for x in &slice_moves {
            for y in &slice_moves {
                if x == y || x == &get_inverse_move(y) {
                    continue;
                }
                let x_inv = get_inverse_move(x);
                let y_inv = get_inverse_move(y);

                let seq = vec![x.clone(), y.clone(), x_inv, y_inv];
                let score = evaluate_seq(&seq)?;
                if score == 3 {
                    return Ok(seq); // Immediate return for perfect move
                } else if score > best_score {
                    best_score = score;
                    best_seq = seq;
                }
            }
        }
    }

    // 2. Try 1-move setup (6-move sequence: S + [X, Y] + S')
    // 2a. Highly optimized: Face Setup + [Slice, Face] / [Face, Slice] commutator
    if best_score < 3 {
        for s in &face_moves {
            let s_inv = get_inverse_move(s);
            for x in &slice_moves {
                for y in &face_moves {
                    if x == y || x == &get_inverse_move(y) {
                        continue;
                    }
                    let x_inv = get_inverse_move(x);
                    let y_inv = get_inverse_move(y);

                    // Form 1: S + [Slice, Face] + S'
                    let seq1 = vec![
                        s.clone(),
                        x.clone(),
                        y.clone(),
                        x_inv.clone(),
                        y_inv.clone(),
                        s_inv.clone(),
                    ];
                    let score1 = evaluate_seq(&seq1)?;
                    if score1 == 3 {
                        return Ok(seq1); // Immediate return for perfect move
                    } else if score1 > best_score {
                        best_score = score1;
                        best_seq = seq1;
                    }

                    // Form 2: S + [Face, Slice] + S'
                    let seq2 = vec![
                        s.clone(),
                        y.clone(),
                        x.clone(),
                        y_inv.clone(),
                        x_inv.clone(),
                        s_inv.clone(),
                    ];
                    let score2 = evaluate_seq(&seq2)?;
                    if score2 == 3 {
                        return Ok(seq2); // Immediate return for perfect move
                    } else if score2 > best_score {
                        best_score = score2;
                        best_seq = seq2;
                    }
                }
            }
        }
    }

    // 2b. Fallback: Any Setup + Any Commutator
    if best_score < 3 {
        // Optimize search space: in dry run, restrict Y to face moves to run 5x faster,
        // while keeping full setup candidate flexibility to solve oblique orbits.
        let simple_y_candidates = if dry_run { &face_moves } else { &y_candidates };

        for s in &all_setup_candidates {
            let s_inv = get_inverse_move(s);
            for x in &slice_moves {
                for y in simple_y_candidates {
                    // Skip if already evaluated in 2a
                    let s_is_face = face_moves.contains(s);
                    let y_is_face = face_moves.contains(y);
                    if s_is_face && y_is_face {
                        continue; // Already checked in 2a
                    }

                    if x == y || x == &get_inverse_move(y) {
                        continue;
                    }
                    let x_inv = get_inverse_move(x);
                    let y_inv = get_inverse_move(y);

                    let seq = vec![s.clone(), x.clone(), y.clone(), x_inv, y_inv, s_inv.clone()];
                    let score = evaluate_seq(&seq)?;
                    if score == 3 {
                        return Ok(seq); // Immediate return for perfect move
                    } else if score > best_score {
                        best_score = score;
                        best_seq = seq;
                    }
                }
            }
        }
    }

    // 3. Try 2-face-move setup (8-move: S1 + S2 + [X, Y] + S2' + S1')
    // Only run this expensive search if we haven't found any perfect solving step (score = 3)
    // and we are NOT in a dry run. Dry runs do not need 8-move commutators as 4-move and 6-move
    // commutators are mathematically sufficient under unconstrained (no global preservation) dry run conditions.
    if best_score < 3 && !dry_run {
        let mut simple_x = Vec::new();
        for m in &slice_moves {
            if !m.ends_with('2') {
                simple_x.push(m.clone());
            }
        }
        let mut simple_y = Vec::new();
        let y_source = &face_moves;
        for m in y_source {
            if !m.ends_with('2') {
                simple_y.push(m.clone());
            }
        }

        for s1 in &face_setup_candidates {
            let s1_inv = get_inverse_move(s1);
            for s2 in &face_setup_candidates {
                if s1 == s2 || s1 == &get_inverse_move(s2) {
                    continue;
                }
                let s2_inv = get_inverse_move(s2);

                for x in &simple_x {
                    for y in &simple_y {
                        if x == y || x == &get_inverse_move(y) {
                            continue;
                        }
                        let x_inv = get_inverse_move(x);
                        let y_inv = get_inverse_move(y);

                        let seq = vec![
                            s1.clone(),
                            s2.clone(),
                            x.clone(),
                            y.clone(),
                            x_inv,
                            y_inv,
                            s2_inv.clone(),
                            s1_inv.clone(),
                        ];
                        let score = evaluate_seq(&seq)?;
                        if score == 3 {
                            return Ok(seq); // Immediate return for perfect move
                        } else if score > best_score {
                            best_score = score;
                            best_seq = seq;
                        }
                    }
                }
            }
        }
    }

    if best_score >= 2 {
        return Ok(best_seq);
    }

    // 4. Strategic Fallback: If no solving commutator was found and we are not in dry_run,
    // perform an expensive rescue scan with y_source = &y_candidates (allowing slice moves as Y)
    // to find a complex commutator that still satisfies the global preservation filter.
    if !dry_run {
        let mut simple_x = Vec::new();
        for m in &slice_moves {
            if !m.ends_with('2') {
                simple_x.push(m.clone());
            }
        }
        let mut simple_y = Vec::new();
        for m in &y_candidates {
            if !m.ends_with('2') {
                simple_y.push(m.clone());
            }
        }

        for s1 in &face_setup_candidates {
            let s1_inv = get_inverse_move(s1);
            for s2 in &face_setup_candidates {
                if s1 == s2 || s1 == &get_inverse_move(s2) {
                    continue;
                }
                let s2_inv = get_inverse_move(s2);

                for x in &simple_x {
                    for y in &simple_y {
                        // Skip if already evaluated (where y is a face move)
                        if face_moves.contains(y) {
                            continue;
                        }
                        if x == y || x == &get_inverse_move(y) {
                            continue;
                        }
                        let x_inv = get_inverse_move(x);
                        let y_inv = get_inverse_move(y);

                        let seq = vec![
                            s1.clone(),
                            s2.clone(),
                            x.clone(),
                            y.clone(),
                            x_inv,
                            y_inv,
                            s2_inv.clone(),
                            s1_inv.clone(),
                        ];
                        let score = evaluate_seq(&seq)?;
                        if score == 3 {
                            return Ok(seq); // Immediate return for perfect move
                        } else if score > best_score {
                            best_score = score;
                            best_seq = seq;
                        }
                    }
                }
            }
        }
    }

    if best_score >= 2 {
        return Ok(best_seq);
    }

    Err(CubeError::InvalidMove(
        "No solving commutator found with positive progress".to_string(),
    ))
}
