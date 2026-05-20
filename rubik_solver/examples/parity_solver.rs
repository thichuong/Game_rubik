// Example demonstrating the Parity Solver for NxN Rubik's Cubes (such as 4x4 and 5x5).
// It shows how to detect OLL Parity (flipped composite edge) and PLL Parity (swapped composite edges),
// how they make the mapped 3x3 state mathematically unsolvable,
// and how applying the respective parity formula moves successfully resolves them.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::uninlined_format_args,
    clippy::missing_const_for_fn
)]

use rubik_solver::core::RotationMove;
use rubik_solver::nxn::parity::{
    apply_oll_parity_to_string, apply_pll_parity_to_string, get_oll_parity_moves,
    get_pll_parity_moves, is_solvable_3x3, map_to_3x3_string,
};
use rubik_solver::nxn::state::NxNState;

// Format a rotation move into a readable string
fn format_move(m: RotationMove) -> String {
    let dir_char = match m.direction {
        rubik_solver::core::Direction::Clockwise => "",
        rubik_solver::core::Direction::CounterClockwise => "'",
    };
    format!("{:?}{}{}", m.axis, m.index, dir_char)
}

fn demonstrate_oll_parity(size: usize) {
    println!("--- [ PART 1: OLL PARITY DEMONSTRATION FOR {size}x{size} ] ---");
    let mut state = NxNState::new(size);

    // 1. Map initial solved state to 3x3 string
    let initial_3x3 = map_to_3x3_string(&state);
    println!("   Initial 3x3 string: {}", initial_3x3);
    println!("   Is solvable?        {}", is_solvable_3x3(&initial_3x3));

    // 2. Apply OLL Parity moves to introduce parity (flip the UF composite edge)
    let oll_moves = get_oll_parity_moves(size);
    println!(
        "   Applying OLL Parity formula moves ({} moves)...",
        oll_moves.len()
    );
    state.apply_moves(&oll_moves);

    // 3. Map new state to 3x3 string
    let parity_3x3 = map_to_3x3_string(&state);
    println!("   Post-OLL parity 3x3 string: {}", parity_3x3);
    let solvable_before_fix = is_solvable_3x3(&parity_3x3);
    println!(
        "   Is solvable now?            {} <-- (Expected: false)",
        solvable_before_fix
    );

    // 4. Resolve the parity by applying the OLL Parity formula moves again
    println!("   Applying OLL Parity formula moves again to solve it...");
    state.apply_moves(&oll_moves);

    // 5. Map final state to 3x3 string
    let solved_3x3 = map_to_3x3_string(&state);
    println!("   Resolved 3x3 string:        {}", solved_3x3);
    let solvable_after_fix = is_solvable_3x3(&solved_3x3);
    println!(
        "   Is solvable after fix?      {} <-- (Expected: true)",
        solvable_after_fix
    );

    // 6. Demonstrate apply_oll_parity_to_string helper
    let direct_fixed_str = apply_oll_parity_to_string(&parity_3x3);
    println!("   Directly fixed 3x3 string:  {}", direct_fixed_str);
    println!(
        "   Is directly fixed solvable? {}",
        is_solvable_3x3(&direct_fixed_str)
    );
    println!("--------------------------------------------------\n");
}

fn demonstrate_pll_parity(size: usize) {
    println!("--- [ PART 2: PLL PARITY DEMONSTRATION FOR {size}x{size} ] ---");
    let mut state = NxNState::new(size);

    // 1. Map initial solved state to 3x3 string
    let initial_3x3 = map_to_3x3_string(&state);
    println!("   Initial 3x3 string: {}", initial_3x3);
    println!("   Is solvable?        {}", is_solvable_3x3(&initial_3x3));

    // 2. Apply PLL Parity moves to introduce parity (swap UF and UB composite edges)
    let pll_moves = get_pll_parity_moves(size);
    println!(
        "   Applying PLL Parity formula moves ({} moves)...",
        pll_moves.len()
    );
    state.apply_moves(&pll_moves);

    // 3. Map new state to 3x3 string
    let parity_3x3 = map_to_3x3_string(&state);
    println!("   Post-PLL parity 3x3 string: {}", parity_3x3);
    let solvable_before_fix = is_solvable_3x3(&parity_3x3);
    println!(
        "   Is solvable now?            {} <-- (Expected: false)",
        solvable_before_fix
    );

    // 4. Resolve the parity by applying the PLL Parity formula moves again
    println!("   Applying PLL Parity formula moves again to solve it...");
    state.apply_moves(&pll_moves);

    // 5. Map final state to 3x3 string
    let solved_3x3 = map_to_3x3_string(&state);
    println!("   Resolved 3x3 string:        {}", solved_3x3);
    let solvable_after_fix = is_solvable_3x3(&solved_3x3);
    println!(
        "   Is solvable after fix?      {} <-- (Expected: true)",
        solvable_after_fix
    );

    // 6. Demonstrate apply_pll_parity_to_string helper
    let direct_fixed_str = apply_pll_parity_to_string(&parity_3x3);
    println!("   Directly fixed 3x3 string:  {}", direct_fixed_str);
    println!(
        "   Is directly fixed solvable? {}",
        is_solvable_3x3(&direct_fixed_str)
    );
    println!("--------------------------------------------------\n");
}

fn print_moves_list(label: &str, moves: &[RotationMove]) {
    print!("{}: ", label);
    for m in moves {
        print!("{} ", format_move(*m));
    }
    println!("\n");
}

fn main() {
    println!("==================================================");
    println!("        NxN RUBIK CUBE PARITY SOLVER DEMO");
    println!("==================================================");

    // Print formulas for awareness
    let oll_moves_4 = get_oll_parity_moves(4);
    let pll_moves_4 = get_pll_parity_moves(4);
    print_moves_list("4x4 OLL Parity Formula", &oll_moves_4);
    print_moves_list("4x4 PLL Parity Formula", &pll_moves_4);

    // Run demonstrations for 4x4
    demonstrate_oll_parity(4);
    demonstrate_pll_parity(4);

    // Run OLL Parity demonstration for 5x5
    demonstrate_oll_parity(5);
}
