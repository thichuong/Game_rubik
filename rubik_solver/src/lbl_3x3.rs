use crate::state::Cube;

pub fn solve_3x3(cube: &mut Cube) -> Vec<String> {
    let mut moves = Vec::new();

    // Stage 1: Cross
    moves.extend(solve_cross(cube));
    // Stage 2: F2L
    moves.extend(solve_f2l(cube));
    // Stage 3: OLL
    moves.extend(solve_oll(cube));
    // Stage 4: PLL
    moves.extend(solve_pll(cube));

    moves
}

const fn solve_cross(_cube: &mut Cube) -> Vec<String> {
    Vec::new()
}

const fn solve_f2l(_cube: &mut Cube) -> Vec<String> {
    Vec::new()
}

const fn solve_oll(_cube: &mut Cube) -> Vec<String> {
    Vec::new()
}

const fn solve_pll(_cube: &mut Cube) -> Vec<String> {
    Vec::new()
}
