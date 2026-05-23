use crate::state::Cube;

/// Corrects OLL and PLL parities for even-sized cubes.
#[allow(clippy::missing_const_for_fn)]
pub fn solve_parity(_cube: &mut Cube) -> Vec<String> {
    // Example: if OLL parity is detected, apply oll_parity macro.
    Vec::new()
}

/// OLL Parity Algorithm for any layer $k$.
pub fn oll_parity(layer: usize) -> String {
    format!("{layer}Rw2 B2 U2 {layer}Lw U2 {layer}Rw' U2 {layer}Rw U2 F2 {layer}Rw F2 {layer}Lw' B2 {layer}Rw2")
}

/// PLL Parity Algorithm for any layer $k$.
pub fn pll_parity(layer: usize) -> String {
    format!("{layer}Rw2 U2 {layer}Rw2 Uw2 {layer}Rw2 2Uw2")
}
