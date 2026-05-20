#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::doc_markdown,
    clippy::option_if_let_else,
    clippy::similar_names
)]

use crate::core::{Face, RotationMove};
use crate::nxn::formulas::{get_oll_parity_fix, get_pll_parity_fix};
use crate::nxn::state::{FACES_ORDER, NxNState};
use kewb::{CubieCube, FaceCube};
use std::collections::HashMap;

/// Maps an NxNState (where centers are solved and edges are paired)
/// into a standard 54-character 3x3x3 state string.
pub fn map_to_3x3_string(state: &NxNState) -> String {
    let size = state.size;
    let mut result = vec![' '; 54];

    // Build a lookup map from (Face, IVec3) to color for O(1) access
    let mut lookup = HashMap::with_capacity(state.facelets.len());
    for f in &state.facelets {
        lookup.insert((f.face, f.coord), f.color);
    }

    let map_idx = |v: usize, is_center: bool| -> usize {
        if v == 0 {
            0
        } else if v == 2 {
            size - 1
        } else {
            // v == 1 (center or edge winglet representation)
            if is_center && size % 2 == 1 {
                size / 2
            } else {
                1
            }
        }
    };

    for (face_idx, &face) in FACES_ORDER.iter().enumerate() {
        for r3 in 0..3 {
            for c3 in 0..3 {
                let is_center = r3 == 1 && c3 == 1;
                let r_n = map_idx(r3, is_center);
                let c_n = map_idx(c3, is_center);

                if let Some(coord) = NxNState::get_logical_coord(face, r_n, c_n, size) {
                    if let Some(&color) = lookup.get(&(face, coord)) {
                        let ch = match color {
                            Face::Up => 'U',
                            Face::Right => 'R',
                            Face::Front => 'F',
                            Face::Down => 'D',
                            Face::Left => 'L',
                            Face::Back => 'B',
                        };
                        result[face_idx * 9 + r3 * 3 + c3] = ch;
                    }
                }
            }
        }
    }
    result.into_iter().collect()
}

/// Returns the sequence of moves to fix OLL Parity (flipped composite edge)
/// Formula: Rw U2 x Rw U2 Rw U2 Rw' U2 Lw U2 Rw' U2 Rw U2 Rw' U2 Rw x'
pub fn get_oll_parity_moves(size: usize) -> Vec<RotationMove> {
    get_oll_parity_fix(size)
}

/// Returns the sequence of moves to fix PLL Parity (swapped composite edges)
/// Formula: r2 U2 r2 Uw2 r2 Uw2
pub fn get_pll_parity_moves(size: usize) -> Vec<RotationMove> {
    get_pll_parity_fix(size)
}

/// Checks if a 3x3 state string is mathematically solvable
pub fn is_solvable_3x3(state_str: &str) -> bool {
    if let Ok(face_cube) = FaceCube::try_from(state_str) {
        CubieCube::try_from(&face_cube).is_ok()
    } else {
        false
    }
}

/// Helper to apply OLL Parity (flip UF edge) directly to the 54-char 3x3 state string.
/// UF edge stickers are at index 7 (Up face) and index 19 (Front face).
pub fn apply_oll_parity_to_string(s: &str) -> String {
    let mut chars: Vec<char> = s.chars().collect();
    if chars.len() == 54 {
        chars.swap(7, 19);
    }
    chars.into_iter().collect()
}

/// Helper to apply PLL Parity (swap UF and UB edges) directly to the 54-char 3x3 state string.
///
/// UF edge stickers are at index 7 (Up) and index 19 (Front).
/// UB edge stickers are at index 1 (Up) and index 46 (Back).
pub fn apply_pll_parity_to_string(s: &str) -> String {
    let mut chars: Vec<char> = s.chars().collect();
    if chars.len() == 54 {
        chars.swap(1, 7);
        chars.swap(19, 46);
    }
    chars.into_iter().collect()
}
