use crate::state::Cube;

#[allow(clippy::missing_const_for_fn)]
pub fn solve_edges(_cube: &mut Cube) -> Vec<String> {
    Vec::new()
}

pub const fn flip_algorithm() -> &'static str {
    "R U R' F R' F' R"
}

/// Applies Slice-Flip-Unslice macro.
pub fn get_slice_flip_unslice(layer: usize) -> String {
    format!("{}Rw {} {}Rw'", layer + 1, flip_algorithm(), layer + 1)
}
