// Example demonstrating the Edge Solver (Edge Pairing) for 4x4 and 5x5 Rubik's Cubes.
// This example scrambles the cube using outer face moves, which rotates the composite edges
// as solid units while keeping their winglets aligned (paired).
// It then calls `pair_edges` to demonstrate that the solver successfully verifies and maintains
// the paired status of all 12 composite edges on the cube.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::uninlined_format_args,
    clippy::missing_const_for_fn
)]

use rubik_solver::core::{Direction, RotationAxis, RotationMove};
use rubik_solver::nxn::edges::pair_edges;
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

// Scramble the cube using outer face moves.
// This rotates the composite edges as solid blocks and keeps the centers completely intact.
fn scramble_outer_faces(
    state: &mut NxNState,
    steps: usize,
    rng: &mut SimpleRng,
) -> Vec<RotationMove> {
    let mut moves = Vec::with_capacity(steps);
    let size = state.size;
    let s = size as i32;

    for _ in 0..steps {
        let axis = match rng.next_range(0, 3) {
            0 => RotationAxis::X,
            1 => RotationAxis::Y,
            _ => RotationAxis::Z,
        };

        let index = match rng.next_range(0, 2) {
            0 => 0,
            _ => s - 1,
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

// Run the outer face scramble and edge pairing demonstration
fn run_demo(size: usize, seed: u64, scramble_steps: usize) {
    println!("==================================================");
    println!("     EDGE SOLVER DEMONSTRATION FOR {size}x{size} CUBE");
    println!("==================================================");

    let mut state = NxNState::new(size);
    let mut rng = SimpleRng::new(seed);

    println!("1. Initial solved state:");
    print_cube_faces(&state);

    println!("2. Scrambling edges with {scramble_steps} outer face moves...");
    println!(
        "   Outer face moves rotate entire composite edges without breaking individual winglets apart."
    );
    let scramble_moves = scramble_outer_faces(&mut state, scramble_steps, &mut rng);

    print!("   Scramble moves applied: ");
    for m in scramble_moves {
        print!("{} ", format_move(m));
    }
    println!("\n");

    println!(
        "3. State after scrambling (Composite edges are relocated, but winglets within each edge remain aligned):"
    );
    print_cube_faces(&state);

    println!("4. Verifying and pairing composite edges...");
    let start_time = std::time::Instant::now();
    let solve_result = pair_edges(&mut state);
    let duration = start_time.elapsed();

    if let Some(solve_moves) = solve_result {
        if solve_moves.is_empty() {
            println!(
                "   [SUCCESS] Edges are already fully paired! No individual winglet swaps needed."
            );
        } else {
            print!("   Moves to pair edges: ");
            for m in &solve_moves {
                print!("{} ", format_move(*m));
            }
            println!();
            println!("   Solved (paired) edges in {} moves.", solve_moves.len());
        }
        println!("   Time taken: {:?}", duration);
        println!("\n5. State after edge pairing solver verification:");
        print_cube_faces(&state);
    } else {
        println!("   [ERROR] Edge pairing failed!");
    }
    println!("==================================================\n");
}

fn main() {
    println!("Starting NxN Rubik Cube Edge Solver Example...\n");

    // Run demonstration for 4x4
    run_demo(4, 42, 5);

    // Run demonstration for 5x5
    run_demo(5, 1337, 5);
}
