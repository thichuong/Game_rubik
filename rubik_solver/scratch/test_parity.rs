use rubik_solver::core::{Direction, RotationAxis, RotationMove};
use rubik_solver::nxn::parity::{is_solvable_3x3, map_to_3x3_string};
use rubik_solver::nxn::state::NxNState;

// Define the new OLL parity with actual rotation 'x'
fn get_new_oll_parity_moves_with_x(size: usize) -> Vec<RotationMove> {
    let s = size as i32;
    let mut moves = Vec::new();

    let rw = |m: &mut Vec<RotationMove>, dir: Direction| {
        m.push(RotationMove {
            axis: RotationAxis::X,
            index: s - 1,
            direction: dir,
            add_to_history: true,
        });
        m.push(RotationMove {
            axis: RotationAxis::X,
            index: s - 2,
            direction: dir,
            add_to_history: true,
        });
    };

    let lw = |m: &mut Vec<RotationMove>, dir: Direction| {
        m.push(RotationMove {
            axis: RotationAxis::X,
            index: 0,
            direction: dir,
            add_to_history: true,
        });
        m.push(RotationMove {
            axis: RotationAxis::X,
            index: 1,
            direction: dir,
            add_to_history: true,
        });
    };

    let u2 = |m: &mut Vec<RotationMove>| {
        let mv = RotationMove {
            axis: RotationAxis::Y,
            index: s - 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        };
        m.push(mv);
        m.push(mv);
    };

    // Rotation x: rotates all slices 0..size around X clockwise
    let x = |m: &mut Vec<RotationMove>| {
        for idx in 0..size {
            m.push(RotationMove {
                axis: RotationAxis::X,
                index: idx as i32,
                direction: Direction::Clockwise,
                add_to_history: true,
            });
        }
    };

    // Rw U2 x Rw U2 Rw U2 Rw' U2 Lw U2 Rw' U2 Rw U2 Rw' U2 Rw
    rw(&mut moves, Direction::Clockwise); // Rw
    u2(&mut moves); // U2
    x(&mut moves); // x
    rw(&mut moves, Direction::Clockwise); // Rw
    u2(&mut moves); // U2
    rw(&mut moves, Direction::Clockwise); // Rw
    u2(&mut moves); // U2
    rw(&mut moves, Direction::CounterClockwise); // Rw'
    u2(&mut moves); // U2
    lw(&mut moves, Direction::Clockwise); // Lw
    u2(&mut moves); // U2
    rw(&mut moves, Direction::CounterClockwise); // Rw'
    u2(&mut moves); // U2
    rw(&mut moves, Direction::Clockwise); // Rw
    u2(&mut moves); // U2
    rw(&mut moves, Direction::CounterClockwise); // Rw'
    u2(&mut moves); // U2
    rw(&mut moves, Direction::Clockwise); // Rw

    moves
}

fn get_new_pll_parity_moves(size: usize) -> Vec<RotationMove> {
    let s = size as i32;
    let mut moves = Vec::new();

    let r2 = |m: &mut Vec<RotationMove>| {
        let mv = RotationMove {
            axis: RotationAxis::X,
            index: s - 2,
            direction: Direction::Clockwise,
            add_to_history: true,
        };
        m.push(mv);
        m.push(mv);
    };

    let u2 = |m: &mut Vec<RotationMove>| {
        let mv = RotationMove {
            axis: RotationAxis::Y,
            index: s - 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        };
        m.push(mv);
        m.push(mv);
    };

    let uw2 = |m: &mut Vec<RotationMove>| {
        let mv = RotationMove {
            axis: RotationAxis::Y,
            index: s - 2,
            direction: Direction::Clockwise,
            add_to_history: true,
        };
        m.push(mv);
        m.push(mv);
    };

    // r2 U2 r2 Uw2 r2 Uw2
    // Wait, the formula is: r2 U2 r2 Uw2 r2 Uw2 (where Uw2 is the double layer U, i.e., index s-1 and s-2).
    // Let's implement Uw2 as rotating BOTH index s-1 and s-2.
    let double_uw2 = |m: &mut Vec<RotationMove>| {
        let mv1 = RotationMove {
            axis: RotationAxis::Y,
            index: s - 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        };
        let mv2 = RotationMove {
            axis: RotationAxis::Y,
            index: s - 2,
            direction: Direction::Clockwise,
            add_to_history: true,
        };
        m.push(mv1);
        m.push(mv1);
        m.push(mv2);
        m.push(mv2);
    };

    // r2
    r2(&mut moves);
    // U2
    u2(&mut moves);
    // r2
    r2(&mut moves);
    // Uw2
    double_uw2(&mut moves);
    // r2
    r2(&mut moves);
    // Uw2
    double_uw2(&mut moves);

    moves
}

fn main() {
    println!("Testing OLL Parity with x...");
    let size = 4;
    let mut state = NxNState::new(size);
    let initial_3x3 = map_to_3x3_string(&state);
    println!("Initial solvable: {}", is_solvable_3x3(&initial_3x3));

    let moves = get_new_oll_parity_moves_with_x(size);
    state.apply_moves(&moves);
    let post_3x3 = map_to_3x3_string(&state);
    println!("Post-OLL solvable: {} (Expected: false)", is_solvable_3x3(&post_3x3));

    // Let's see what has changed by comparing to initial
    println!("Initial:  {}", initial_3x3);
    println!("Post-OLL: {}", post_3x3);

    // Apply again to see if it solves
    state.apply_moves(&moves);
    let resolved_3x3 = map_to_3x3_string(&state);
    println!("Resolved solvable: {} (Expected: true)", is_solvable_3x3(&resolved_3x3));
    println!("Resolved: {}", resolved_3x3);

    println!("\nTesting PLL Parity...");
    let mut state_pll = NxNState::new(size);
    let pll_moves = get_new_pll_parity_moves(size);
    state_pll.apply_moves(&pll_moves);
    let post_pll_3x3 = map_to_3x3_string(&state_pll);
    println!("Post-PLL solvable: {} (Expected: false)", is_solvable_3x3(&post_pll_3x3));
    println!("Post-PLL: {}", post_pll_3x3);

    state_pll.apply_moves(&pll_moves);
    let resolved_pll_3x3 = map_to_3x3_string(&state_pll);
    println!("Resolved PLL solvable: {} (Expected: true)", is_solvable_3x3(&resolved_pll_3x3));
}
