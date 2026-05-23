pub mod centers;
pub mod edges;
pub mod parity;

use crate::state::Cube;

pub fn solve_reduction(cube: &mut Cube) -> Vec<String> {
    let mut all_moves = Vec::new();
    all_moves.extend(centers::solve_centers(cube));
    all_moves.extend(edges::solve_edges(cube));
    all_moves.extend(parity::solve_parity(cube));
    all_moves
}
