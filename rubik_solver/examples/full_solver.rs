// Example demonstrating a complete end-to-end solver for NxN Rubik's Cubes (such as 4x4 and 5x5):
// 1. Centers Scrambling (dispersing center pieces using scramble_centers).
// 2. Reduction Stage:
//    - Stage A: Solving centers using solve_centers.
//    - Stage B: Pairing composite edges using pair_edges.
// 3. Parity Resolution:
//    - Map state to 3x3 string.
//    - Identify and apply OLL and/or PLL Parity formulas if necessary.
// 4. Kociemba 3x3 Solver Stage:
//    - Solve the 3x3 reduction state using the 2-phase Kociemba solver.
//    - Translate 3x3 moves into NxN physical rotation moves.
// 5. Final Verification: Check if the cube is fully solved (100% success).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::uninlined_format_args,
    clippy::missing_const_for_fn,
    clippy::similar_names,
    clippy::too_many_lines
)]

use rubik_solver::core::{Direction, FaceMapping, RotationAxis, RotationMove};
use rubik_solver::nxn::centers::solve_centers;
use rubik_solver::nxn::edges::pair_edges;
use rubik_solver::nxn::parity::{
    apply_oll_parity_to_string, apply_pll_parity_to_string, get_oll_parity_moves,
    get_pll_parity_moves, is_solvable_3x3, map_to_3x3_string,
};
use rubik_solver::nxn::state::NxNState;

// Simple LCG PRNG for deterministic scrambles without external dependencies
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        (self.state >> 32) as u32
    }

    fn next_range(&mut self, min: usize, max: usize) -> usize {
        let range = max - min;
        if range == 0 {
            return min;
        }
        min + (self.next_u32() as usize % range)
    }
}

// Scramble the cube's inner slices to disperse the center pieces.
// For odd-sized cubes, we avoid turning the middle slice to keep fixed centers in place.
fn scramble_centers(state: &mut NxNState, steps: usize, rng: &mut SimpleRng) -> Vec<RotationMove> {
    let mut moves = Vec::with_capacity(steps);
    let size = state.size;

    for _ in 0..steps {
        let axis = match rng.next_range(0, 3) {
            0 => RotationAxis::X,
            1 => RotationAxis::Y,
            _ => RotationAxis::Z,
        };

        let index = if size > 2 {
            let mut idx = rng.next_range(1, size - 1) as i32;
            if size % 2 == 1 {
                let mid = (size / 2) as i32;
                while idx == mid {
                    idx = rng.next_range(1, size - 1) as i32;
                }
            }
            idx
        } else {
            0
        };

        let direction = match rng.next_range(0, 2) {
            0 => Direction::Clockwise,
            _ => Direction::CounterClockwise,
        };

        let m = RotationMove {
            axis,
            index,
            direction,
            add_to_history: true,
        };
        state.apply_move(m);
        moves.push(m);
    }
    moves
}

// Format a rotation move into a readable string
fn format_move(m: RotationMove) -> String {
    let dir_char = match m.direction {
        Direction::Clockwise => "",
        Direction::CounterClockwise => "'",
    };
    format!("{:?}{}{}", m.axis, m.index, dir_char)
}

// Render the 6 faces of the NxN cube in the console
fn print_cube_faces(state: &NxNState) {
    let size = state.size;
    let str_rep = state.to_string_rep();
    let bytes = str_rep.as_bytes();
    let faces = [
        "Up (White)",
        "Right (Red)",
        "Front (Green)",
        "Down (Yellow)",
        "Left (Orange)",
        "Back (Blue)",
    ];

    for (face_idx, face_name) in faces.iter().enumerate() {
        println!("  [ {} ]", face_name);
        for row in 0..size {
            print!("    ");
            for col in 0..size {
                let idx = face_idx * size * size + row * size + col;
                if idx < bytes.len() {
                    print!("{} ", bytes[idx] as char);
                } else {
                    print!("? ");
                }
            }
            println!();
        }
        println!();
    }
}

// Run the full reduction, parity fix, and 3x3 solver pipeline
fn run_full_solver_demo(size: usize, seed: u64, scramble_steps: usize, table: &kewb::DataTable) {
    println!("==================================================");
    println!("       FULL SOLVER FOR {size}x{size} RUBIK CUBE");
    println!("==================================================");

    let mut state = NxNState::new(size);
    let mut rng = SimpleRng::new(seed);

    // 1. Scramble Centers
    println!("1. Scrambling centers with {scramble_steps} moves...");
    let scramble_moves = scramble_centers(&mut state, scramble_steps, &mut rng);
    print!("   Scramble moves: ");
    for m in scramble_moves {
        print!("{} ", format_move(m));
    }
    println!("\n");

    println!("--- [ State after Scrambling ] ---");
    print_cube_faces(&state);

    // 2. Reduction Stage A: Solve Centers
    println!("2. Solving centers...");
    let start_centers = std::time::Instant::now();
    let Some(center_moves) = solve_centers(&mut state) else {
        println!("   [ERROR] Failed to solve centers!");
        println!("==================================================\n");
        return;
    };
    println!(
        "   [SUCCESS] Centers solved in {} moves. (Time: {:?})",
        center_moves.len(),
        start_centers.elapsed()
    );

    println!("--- [ State after Stage A (Centers Solved) ] ---");
    print_cube_faces(&state);

    // 3. Reduction Stage B: Pair Edges
    println!("3. Pairing composite edges...");
    let start_edges = std::time::Instant::now();
    let Some(edge_moves) = pair_edges(&mut state) else {
        println!("   [ERROR] Failed to pair edges!");
        println!("==================================================\n");
        return;
    };
    println!(
        "   [SUCCESS] Edges paired in {} moves. (Time: {:?})",
        edge_moves.len(),
        start_edges.elapsed()
    );

    println!("--- [ State after Stage B (Edges Paired) ] ---");
    print_cube_faces(&state);

    // 4. Parity Identification & Resolution
    println!("4. Verifying and solving Parity...");
    let base_3x3_str = map_to_3x3_string(&state);
    let combo_attempts = [
        (false, false), // No parities
        (true, false),  // OLL parity only
        (false, true),  // PLL parity only
        (true, true),   // Both OLL and PLL parities
    ];

    let mut best_combo = None;
    let mut final_3x3_state_str = String::new();

    for &(try_oll, try_pll) in &combo_attempts {
        let mut temp_3x3 = base_3x3_str.clone();
        if try_oll {
            temp_3x3 = apply_oll_parity_to_string(&temp_3x3);
        }
        if try_pll {
            temp_3x3 = apply_pll_parity_to_string(&temp_3x3);
        }

        if is_solvable_3x3(&temp_3x3) {
            best_combo = Some((try_oll, try_pll));
            final_3x3_state_str = temp_3x3;
            break;
        }
    }

    let Some((need_oll, need_pll)) = best_combo else {
        println!("   [ERROR] Could not find a solvable 3x3 mapping!");
        println!("==================================================\n");
        return;
    };

    println!(
        "   Parity status -> OLL Parity needed: {}, PLL Parity needed: {}",
        need_oll, need_pll
    );

    let mut parity_moves = Vec::new();
    if need_oll {
        parity_moves.extend(get_oll_parity_moves(size));
    }
    if need_pll {
        parity_moves.extend(get_pll_parity_moves(size));
    }

    if parity_moves.is_empty() {
        println!("   No parity correction needed.");
    } else {
        println!(
            "   Applying parity correction ({} moves)...",
            parity_moves.len()
        );
        state.apply_moves(&parity_moves);
        println!("--- [ State after Parity Correction ] ---");
        print_cube_faces(&state);
    }

    // 5. Kociemba 3x3 Solver Stage
    println!("5. Solving the reduced 3x3 state using Kociemba solver...");
    let start_3x3 = std::time::Instant::now();
    let Ok(face_cube) = kewb::FaceCube::try_from(final_3x3_state_str.as_str()) else {
        println!("   [ERROR] Failed to convert solvable 3x3 state to FaceCube!");
        println!("==================================================\n");
        return;
    };
    let Ok(cubie_cube) = kewb::CubieCube::try_from(&face_cube) else {
        println!("   [ERROR] Failed to convert FaceCube to CubieCube!");
        println!("==================================================\n");
        return;
    };
    let mut solver = kewb::Solver::new(table, 23, None);
    let Some(sol) = solver.solve(cubie_cube) else {
        println!("   [ERROR] Kociemba solver failed to solve the 3x3 state!");
        println!("==================================================\n");
        return;
    };

    let sol_str = sol.to_string();
    println!("   Kociemba 3x3 logic moves: {}", sol_str);

    // Translate logic moves to NxN physical moves
    let mapping = FaceMapping::default();
    let physical_3x3_moves =
        rubik_solver::helpers::logical_string_to_physical_moves_any(&sol_str, size as i32, mapping);
    state.apply_moves(&physical_3x3_moves);
    println!(
        "   [SUCCESS] Reduced 3x3 solved in {} moves. (Time: {:?})",
        physical_3x3_moves.len(),
        start_3x3.elapsed()
    );

    // 6. Final Verification
    println!("6. Final Verification Stage...");
    let final_3x3_str = map_to_3x3_string(&state);
    let solved_target = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
    if final_3x3_str == solved_target {
        println!("   [VERIFICATION SUCCESS] The Rubik's cube has been fully solved 100%!");
    } else {
        println!("   [VERIFICATION FAILED] State mismatch!");
        println!("   Expected: {}", solved_target);
        println!("   Got:      {}", final_3x3_str);
    }

    println!("--- [ Final Fully Solved State ] ---");
    print_cube_faces(&state);

    println!("==================================================\n");
}

fn main() {
    println!("Loading Kociemba 2-phase data table...");
    let table = kewb::DataTable::default();
    println!("Data table successfully loaded!\n");

    // Test full solver for 4x4 with complex center scramble
    run_full_solver_demo(4, 42, 10, &table);

    // Test full solver for 5x5 with complex center scramble
    run_full_solver_demo(5, 1337, 10, &table);
}
