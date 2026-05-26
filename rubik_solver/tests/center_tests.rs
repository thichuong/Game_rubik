#![allow(clippy::needless_range_loop)]

// Integration and unit tests for the nxn Rubik's cube center solver.
// All comments in source files must be in English.

use rubik_solver::Cube;
use rubik_solver::center_solver::center::solve_centers;
use rubik_solver::center_solver::orbit::decompose_orbits;
use rubik_solver::cube::Face;

/// Utility to generate a random scramble path for an nxn cube.
fn scramble_cube(cube: &mut Cube, depth: usize) -> Vec<String> {
    let size = cube.size();
    let outer_faces = ["U", "D", "F", "B", "L", "R"];
    let slice_types = ['u', 'd', 'r', 'l', 'f', 'b'];
    let modifiers = ["", "'", "2"];

    let mut scramble_moves = Vec::new();

    // Simple pseudo-random generator
    let mut seed = 12345usize;
    let mut rand = || {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        seed & 0x7FFFFFFF
    };

    for _ in 0..depth {
        let is_slice = rand() % 2 == 0 && size >= 4;
        let m = if is_slice {
            let s_type = slice_types[rand() % slice_types.len()];
            let slice_idx = (rand() % (size - 2)) + 1; // 1 to size-2
            let modifier = modifiers[rand() % modifiers.len()];
            format!("{}{}{}", s_type, slice_idx, modifier)
        } else {
            let face = outer_faces[rand() % outer_faces.len()];
            let modifier = modifiers[rand() % modifiers.len()];
            format!("{}{}", face, modifier)
        };

        // Apply and save move
        if cube.apply_move(&m).is_ok() {
            scramble_moves.push(m);
        }
    }

    scramble_moves
}

/// Helper to verify that all mobile centers in the cube are solved.
fn assert_centers_solved(cube: &Cube) {
    let size = cube.size();
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
                // Skip the central fixed center of odd cubes
                if size % 2 == 1 && r == size / 2 && c == size / 2 {
                    continue;
                }
                let val = cube.get(face, r, c).unwrap();
                assert_eq!(
                    val, face,
                    "Mobile center piece at {:?}(row: {}, col: {}) was expected to be {:?}, but got {:?}",
                    face, r, c, face, val
                );
            }
        }
    }
}

#[test]
fn test_cube_creation_and_moves() {
    let mut cube = Cube::new(6).unwrap();
    assert_eq!(cube.size(), 6);

    // Check initial solved state
    assert_eq!(cube.get(Face::U, 1, 1).unwrap(), Face::U);
    assert_eq!(cube.get(Face::F, 3, 3).unwrap(), Face::F);

    // Apply some turns and check they compile and run safely
    assert!(cube.apply_move("U").is_ok());
    assert!(cube.apply_move("r1'").is_ok());
    assert!(cube.apply_move("d32").is_ok());
}

#[test]
fn test_rotation_periodicity() {
    let cube = Cube::new(6).unwrap();
    let initial_state = cube.to_string_state();

    let moves_to_test = [
        "U", "D", "F", "B", "L", "R", "r1", "l1", "u1", "d1", "f1", "b1",
    ];
    for &m in &moves_to_test {
        let mut test_cube = Cube::new(6).unwrap();
        for _ in 0..4 {
            test_cube.apply_move(m).unwrap();
        }
        assert_eq!(
            test_cube.to_string_state(),
            initial_state,
            "Move {} 4 times did not restore state",
            m
        );
    }
}

#[test]
fn test_single_commutator_solve() {
    let mut cube = Cube::new(6).unwrap();
    let initial_state = cube.to_string_state();

    // Apply a simple 3-cycle commutator to scramble exactly 3 pieces in orbit (1, 1, 0)
    // Formula: r1 u1 r1' u1'
    let comm = ["r1", "u1", "r1'", "u1'"];
    println!("Applying diagnostic commutator: {:?}", comm);
    for m in &comm {
        cube.apply_move(m).unwrap();
    }

    let scrambled_state = cube.to_string_state();
    assert_ne!(
        scrambled_state, initial_state,
        "Cube should be scrambled by the commutator"
    );

    // Now solve it!
    let solve_moves = solve_centers(&mut cube).unwrap();
    println!("Scrambled state solved with moves: {:?}", solve_moves);

    assert_eq!(
        cube.to_string_state(),
        initial_state,
        "Cube should be fully solved back to initial state"
    );
}

#[test]
fn test_oblique_commutator_solve() {
    let mut cube = Cube::new(6).unwrap();
    let initial_state = cube.to_string_state();

    // Apply a simple [Slice, Face] commutator to scramble exactly 3 pieces in oblique orbit (1, 2, 0)
    // Formula: r1 U r1' U'
    let comm = ["r1", "U", "r1'", "U'"];
    println!("Applying diagnostic oblique commutator: {:?}", comm);
    for m in &comm {
        cube.apply_move(m).unwrap();
    }

    let scrambled_state = cube.to_string_state();
    assert_ne!(
        scrambled_state, initial_state,
        "Cube should be scrambled by the commutator"
    );

    // Now solve it!
    let solve_moves = solve_centers(&mut cube).unwrap();
    println!("Scrambled state solved with moves: {:?}", solve_moves);

    assert_centers_solved(&cube);
}

#[test]
fn test_commutator_3_cycle_movement() {
    use rubik_solver::center_solver::commutator::find_3cycle_commutator;

    let mut cube = Cube::new(6).unwrap();
    let initial_state = cube.to_string_state();

    // Apply a known 4-move commutator to scramble exactly 3 pieces in orbit (1, 1, 0)
    let scramble_seq = vec![
        "r1".to_string(),
        "u1".to_string(),
        "r1'".to_string(),
        "u1'".to_string(),
    ];
    for m in &scramble_seq {
        cube.apply_move(m).unwrap();
    }

    let orbits = decompose_orbits(6);
    let orbit = orbits
        .iter()
        .find(|o| o.d_min == 1 && o.d_max == 1)
        .ok_or("Orbit (1, 1) not found")
        .unwrap();

    // Find the pieces that were scrambled
    let mut scrambled_pieces = Vec::new();
    for p in &orbit.pieces {
        let label = cube.get(p.face, p.row, p.col).unwrap();
        if label != p.face {
            scrambled_pieces.push(*p);
        }
    }

    println!("Scrambled pieces in orbit (1,1,0): {:?}", scrambled_pieces);
    assert!(
        scrambled_pieces.len() >= 3,
        "Scramble must affect at least 3 pieces"
    );

    // Let's identify the 3-cycle: piece_a -> piece_b -> piece_c -> piece_a
    // Since we scrambled with a 3-cycle, there must be exactly some 3 pieces involved.
    let piece_a = scrambled_pieces[0];
    let target_a = cube.get(piece_a.face, piece_a.row, piece_a.col).unwrap();

    // Find piece_b which is currently at the target face of A
    let mut piece_b = piece_a;
    for &p in &scrambled_pieces {
        if p.face == target_a {
            piece_b = p;
            break;
        }
    }

    let target_b = cube.get(piece_b.face, piece_b.row, piece_b.col).unwrap();
    // Find piece_c which is currently at the target face of B
    let mut piece_c = piece_a;
    for &p in &scrambled_pieces {
        if p.face == target_b && p != piece_a && p != piece_b {
            piece_c = p;
            break;
        }
    }

    println!(
        "Identified 3-cycle: A={:?} (has color {:?}) -> B={:?} (has color {:?}) -> C={:?}",
        piece_a, target_a, piece_b, target_b, piece_c
    );

    let len = orbit.pieces.len();
    let mut solved_mask = vec![true; len];
    for i in 0..len {
        let p = &orbit.pieces[i];
        if scrambled_pieces.contains(p) {
            solved_mask[i] = false;
        }
    }

    // Now find the commutator to solve: A -> B -> C -> A
    let seq = find_3cycle_commutator(6, &cube, orbit, &piece_a, &piece_b, &piece_c, &solved_mask)
        .unwrap();
    println!("Found solving commutator sequence: {:?}", seq);

    // Apply the solving sequence
    for m in &seq {
        cube.apply_move(m).unwrap();
    }

    // Check if the cube is fully solved back
    assert_eq!(
        cube.to_string_state(),
        initial_state,
        "Applying the found commutator should solve the cube"
    );
    println!("SUCCESS: Commutator correctly found and solved the 3 scrambled pieces!");
}

#[test]
fn test_orbit_decomposition() {
    // 6x6 Cube
    // Center grid size: 4x4 = 16 pieces per face. Total mobile centers: 16 * 6 = 96 pieces.
    // Since n=6 is even:
    // Orbit 1: Diagonal outer (d_min: 1, d_max: 1) -> 24 pieces
    // Orbit 2: Diagonal inner (d_min: 2, d_max: 2) -> 24 pieces
    // Orbit 3: Oblique (d_min: 1, d_max: 2, sub_orbit 0) -> 48 pieces (Group A & B merged)
    // Total: 3 orbits = 96 pieces.
    let orbits_6x6 = decompose_orbits(6);
    assert_eq!(orbits_6x6.len(), 3);
    let mut total_pieces = 0;
    for o in &orbits_6x6 {
        assert!(o.pieces.len() == 24 || o.pieces.len() == 48);
        total_pieces += o.pieces.len();
    }
    assert_eq!(total_pieces, 96);

    // 7x7 Cube
    // Center grid size: 5x5 = 25 pieces per face.
    // Central piece (row: 3, col: 3) is fixed. Total mobile centers: 24 * 6 = 144 pieces.
    // Orbits:
    // 1. Diagonal outer (1, 1) -> 24
    // 2. Diagonal inner (2, 2) -> 24
    // 3. Oblique (1, 2) -> 48 (merged)
    // 4. Cross outer (1, 3) -> 24
    // 5. Cross inner (2, 3) -> 24
    // Total: 5 orbits = 144 pieces.
    let orbits_7x7 = decompose_orbits(7);
    assert_eq!(orbits_7x7.len(), 5);
    let mut total_pieces = 0;
    for o in &orbits_7x7 {
        assert!(o.pieces.len() == 24 || o.pieces.len() == 48);
        total_pieces += o.pieces.len();
    }
    assert_eq!(total_pieces, 144);
}

#[test]
fn test_integration_solve_centers_6x6_depth_5() {
    let mut cube = Cube::new(6).unwrap();
    let scramble = scramble_cube(&mut cube, 5);
    println!("Scramble 6x6 (depth 5): {:?}", scramble);

    let solve_moves = solve_centers(&mut cube).unwrap();
    println!("Solve moves: {:?}", solve_moves);

    assert_centers_solved(&cube);
}

#[test]
fn test_integration_solve_centers_6x6_depth_10() {
    let mut cube = Cube::new(6).unwrap();
    let scramble = scramble_cube(&mut cube, 10);
    println!("Scramble 6x6 (depth 10): {:?}", scramble);

    let solve_moves = solve_centers(&mut cube).unwrap();
    println!("Solve moves: {:?}", solve_moves);

    assert_centers_solved(&cube);
}

#[test]
fn test_integration_solve_centers_6x6_depth_25() {
    let mut cube = Cube::new(6).unwrap();
    let scramble = scramble_cube(&mut cube, 25);
    println!("Scramble 6x6 (depth 25): {:?}", scramble);

    let solve_moves = solve_centers(&mut cube).unwrap();
    println!("Solve moves: {:?}", solve_moves);

    assert_centers_solved(&cube);
}

#[test]
fn test_integration_solve_centers_6x6_depth_50() {
    let mut cube = Cube::new(6).unwrap();
    let scramble = scramble_cube(&mut cube, 50);
    println!("Scramble 6x6 (depth 50): {:?}", scramble);

    let solve_moves = solve_centers(&mut cube).unwrap();
    println!("Solve moves: {:?}", solve_moves);

    assert_centers_solved(&cube);
}

#[test]
fn test_integration_solve_centers_6x6_depth_100() {
    let mut cube = Cube::new(6).unwrap();
    let scramble = scramble_cube(&mut cube, 100);
    println!("Scramble 6x6 (depth 100): {:?}", scramble);

    let solve_moves = solve_centers(&mut cube).unwrap();
    println!("Solve moves: {:?}", solve_moves);

    assert_centers_solved(&cube);
}

#[test]
fn test_integration_solve_centers_7x7_depth_5() {
    let mut cube = Cube::new(7).unwrap();
    let scramble = scramble_cube(&mut cube, 5);
    println!("Scramble 7x7 (depth 5): {:?}", scramble);

    let solve_moves = solve_centers(&mut cube).unwrap();
    println!("Solve moves: {:?}", solve_moves);

    assert_centers_solved(&cube);
}

#[test]
fn test_integration_solve_centers_7x7_depth_10() {
    let mut cube = Cube::new(7).unwrap();
    let scramble = scramble_cube(&mut cube, 10);
    println!("Scramble 7x7 (depth 10): {:?}", scramble);

    let solve_moves = solve_centers(&mut cube).unwrap();
    println!("Solve moves: {:?}", solve_moves);

    assert_centers_solved(&cube);
}

#[test]
fn test_integration_solve_centers_7x7_depth_25() {
    let mut cube = Cube::new(7).unwrap();
    let scramble = scramble_cube(&mut cube, 25);
    println!("Scramble 7x7 (depth 25): {:?}", scramble);

    let solve_moves = solve_centers(&mut cube).unwrap();
    println!("Solve moves: {:?}", solve_moves);

    assert_centers_solved(&cube);
}

#[test]
fn test_integration_solve_centers_7x7_depth_50() {
    let mut cube = Cube::new(7).unwrap();
    let scramble = scramble_cube(&mut cube, 50);
    println!("Scramble 7x7 (depth 50): {:?}", scramble);

    let solve_moves = solve_centers(&mut cube).unwrap();
    println!("Solve moves: {:?}", solve_moves);

    assert_centers_solved(&cube);
}

#[test]
fn test_integration_solve_centers_7x7_depth_100() {
    let mut cube = Cube::new(7).unwrap();
    let scramble = scramble_cube(&mut cube, 100);
    println!("Scramble 7x7 (depth 100): {:?}", scramble);

    let solve_moves = solve_centers(&mut cube).unwrap();
    println!("Solve moves: {:?}", solve_moves);

    assert_centers_solved(&cube);
}
