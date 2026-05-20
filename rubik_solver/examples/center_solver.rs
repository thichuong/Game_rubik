// Example demonstrating the Center Solver for 4x4 and 5x5 Rubik's Cubes.
// This example scrambles the cube's centers and then solves them,
// printing each stage, the moves taken, and the visual state of the 6 faces.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::uninlined_format_args,
    clippy::missing_const_for_fn
)]

use rubik_solver::core::{Direction, RotationAxis, RotationMove};
use rubik_solver::nxn::centers::solve_centers;
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

// Scramble the cube's inner slices to disperse the center pieces
fn scramble_centers(state: &mut NxNState, steps: usize, rng: &mut SimpleRng) -> Vec<RotationMove> {
    let mut moves = Vec::with_capacity(steps);
    let size = state.size;

    for _ in 0..steps {
        let axis = match rng.next_range(0, 3) {
            0 => RotationAxis::X,
            1 => RotationAxis::Y,
            _ => RotationAxis::Z,
        };
        // Only turn inner slices (1 to size-2) to explicitly scramble the centers.
        // For odd-sized cubes, avoid turning the middle slice to keep the fixed centers in place.
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

// Run the scramble & solve demonstration for a specific size
fn run_demo(size: usize, seed: u64) {
    println!("==================================================");
    println!("          DEMONSTRATION FOR {size}x{size} CUBE");
    println!("==================================================");

    let mut state = NxNState::new(size);
    let mut rng = SimpleRng::new(seed);

    println!("1. Initial solved state:");
    print_cube_faces(&state);

    println!("2. Scrambling the centers...");
    // Scramble the centers with 60 inner slice moves as requested by the user
    let scramble_moves = scramble_centers(&mut state, 60, &mut rng);

    print!("   Scramble move applied: ");
    for m in scramble_moves {
        print!("{} ", format_move(m));
    }
    println!("\n");

    println!("3. State after scrambling (Notice the mixed centers):");
    print_cube_faces(&state);

    println!("4. Solving the centers...");
    let start_time = std::time::Instant::now();
    let solve_result = solve_centers(&mut state);
    let duration = start_time.elapsed();

    if let Some(solve_moves) = solve_result {
        if solve_moves.is_empty() {
            println!("   Centers are already solved! No moves needed.");
        } else {
            print!("   Moves to solve centers: ");
            for m in &solve_moves {
                print!("{} ", format_move(*m));
            }
            println!();
            println!("   Solved in {} moves.", solve_moves.len());
        }
        println!("   Time taken: {:?}", duration);
        println!("\n5. State after solving centers successfully:");
        print_cube_faces(&state);
    } else {
        println!("   [ERROR] Failed to solve centers!");
    }
    println!("==================================================\n");
}

fn main() {
    println!("Starting NxN Rubik Cube Center Solver Example...\n");

    // Run demonstration for 4x4
    run_demo(4, 42);

    // Run demonstration for 5x5
    run_demo(5, 1337);
}
