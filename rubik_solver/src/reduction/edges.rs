use crate::state::Cube;

/// Edge pairing using the Slice-Flip-Unslice technique.
#[allow(clippy::missing_const_for_fn)]
pub fn solve_edges(_cube: &mut Cube) -> Vec<String> {
    // Example logic:
    // 1. Find edge pieces that match.
    // 2. Align them to the front-left and front-right slots.
    // 3. Apply the slice-flip-unslice macro.
    Vec::new()
}

pub const fn flip_algorithm() -> &'static str {
    "R U R' F R' F' R"
}

/// Applies Slice-Flip-Unslice macro.
/// [slice, flip] = slice flip slice'
pub fn get_slice_flip_unslice(layer: usize) -> String {
    format!("{}Rw {} {}Rw'", layer + 1, flip_algorithm(), layer + 1)
}
