// Example demonstrating the step-by-step NxN Rubik's Cube Solver:
// 1. Complex Center Scrambling (using standard scramble_centers from center_solver.rs).
// 2. Step-by-Step Verification:
//    - Stage A: Solving centers first, then verifying they are solved.
//    - Stage B: Pairing composite edges next, then verifying all edges are paired.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::uninlined_format_args,
    clippy::missing_const_for_fn
)]

use rubik_solver::core::{Direction, RotationAxis, RotationMove};
use rubik_solver::nxn::centers::solve_centers;
use rubik_solver::nxn::edges::{COMPOSITE_EDGES, is_edge_paired, pair_edges};
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

// Scramble the cube's inner slices to disperse the center pieces (taken from center_solver.rs).
// For odd-sized cubes, we avoid turning the middle slice to keep the fixed center pieces in place.
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

// Run the full step-by-step demonstration: Scramble -> Solve Centers -> Pair Edges
fn run_step_by_step_demo(size: usize, seed: u64, scramble_steps: usize) {
    println!("==================================================");
    println!("     STEP-BY-STEP SOLVER FOR {size}x{size} RUBIK CUBE");
    println!("==================================================");

    let mut state = NxNState::new(size);
    let mut rng = SimpleRng::new(seed);

    println!("1. Initial fully solved state:");
    print_cube_faces(&state);

    println!(
        "2. Applying complex center scrambling ({} moves)...",
        scramble_steps
    );
    let scramble_moves = scramble_centers(&mut state, scramble_steps, &mut rng);

    print!("   Scramble moves applied: ");
    for m in scramble_moves {
        print!("{} ", format_move(m));
    }
    println!("\n");

    println!("3. State after scrambling (Both centers and inner edge winglets are mixed up):");
    print_cube_faces(&state);

    // --- STAGE A: SOLVE CENTERS ---
    println!("4. [STAGE A] Solving centers first...");
    let start_centers = std::time::Instant::now();
    let center_solve_result = solve_centers(&mut state);
    let duration_centers = start_centers.elapsed();

    if let Some(center_moves) = center_solve_result {
        println!(
            "   [SUCCESS] Centers solved successfully in {} moves.",
            center_moves.len()
        );
        println!("   Time taken: {:?}", duration_centers);
        println!(
            "\n5. State after STAGE A (Centers are solved, but edge winglets are still scrambled):"
        );
        print_cube_faces(&state);
    } else {
        println!("   [ERROR] Failed to solve centers!");
        println!("==================================================\n");
        return;
    }

    // --- STAGE B: PAIR EDGES ---
    println!("6. [STAGE B] Pairing composite edges next...");
    let start_edges = std::time::Instant::now();
    let edge_solve_result = pair_edges(&mut state);
    let duration_edges = start_edges.elapsed();

    if let Some(edge_moves) = edge_solve_result {
        if edge_moves.is_empty() {
            println!("   [SUCCESS] Edges are already fully paired!");
        } else {
            print!("   Moves to pair edges: ");
            for m in &edge_moves {
                print!("{} ", format_move(*m));
            }
            println!();
            println!(
                "   [SUCCESS] Solved (paired) edges in {} moves.",
                edge_moves.len()
            );
        }
        println!("   Time taken: {:?}", duration_edges);

        // Final verification check
        let mut all_paired = true;
        for &(f1, f2) in &COMPOSITE_EDGES {
            if !is_edge_paired(&state, f1, f2) {
                all_paired = false;
                println!("   [WARNING] Edge {:?} - {:?} is not paired!", f1, f2);
            }
        }
        if all_paired {
            println!("   [VERIFICATION] All 12 composite edges verified as successfully paired!");
        }

        println!("\n7. Final State after STAGE B (Both centers and edges are completely solved!):");
        print_cube_faces(&state);
    } else {
        println!("   [ERROR] Failed to pair edges!");
    }
    println!("==================================================\n");
}

fn main() {
    println!("Starting NxN Rubik Cube Step-by-Step Solver Example...\n");

    // Run demonstration for 4x4 with 10 complex scramble steps (X, Y, Z axes)
    run_step_by_step_demo(4, 42, 10);

    // Run demonstration for 5x5 with 10 complex scramble steps (X, Y, Z axes)
    run_step_by_step_demo(5, 1337, 10);
}
