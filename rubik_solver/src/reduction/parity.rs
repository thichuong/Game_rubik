use crate::state::Cube;

pub const fn solve_parity(_cube: &mut Cube) -> Vec<String> {
    Vec::new()
}

pub fn oll_parity(layer: usize) -> String {
    format!("{layer}Rw2 B2 U2 {layer}Lw U2 {layer}Rw' U2 {layer}Rw U2 F2 {layer}Rw F2 {layer}Lw' B2 {layer}Rw2")
}

pub fn pll_parity(layer: usize) -> String {
    format!("{layer}Rw2 U2 {layer}Rw2 Uw2 {layer}Rw2 2Uw2")
}
