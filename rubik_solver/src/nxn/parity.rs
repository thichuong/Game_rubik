#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::doc_markdown,
    clippy::option_if_let_else,
    clippy::similar_names
)]

use crate::core::{Direction, Face, RotationAxis, RotationMove};
use crate::nxn::state::{FACES_ORDER, NxNState};
use kewb::{CubieCube, FaceCube};

/// Maps an NxNState (where centers are solved and edges are paired)
/// into a standard 54-character 3x3x3 state string.
pub fn map_to_3x3_string(state: &NxNState) -> String {
    let size = state.size;
    let mut result = vec![' '; 54];

    let map_idx = |v: usize| -> usize {
        if v == 0 {
            0
        } else if v == 1 {
            1
        } else {
            size - 1
        }
    };

    for (face_idx, &face) in FACES_ORDER.iter().enumerate() {
        for r3 in 0..3 {
            for c3 in 0..3 {
                let r_n = map_idx(r3);
                let c_n = map_idx(c3);

                if let Some(coord) = NxNState::get_logical_coord(face, r_n, c_n, size) {
                    if let Some(facelet) = state
                        .facelets
                        .iter()
                        .find(|f| f.coord == coord && f.face == face)
                    {
                        let ch = match facelet.color {
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
/// Formula: Rw2 B2 U2 Lw U2 Rw' U2 Rw U2 F2 Rw F2 Lw' B2 Rw2
pub fn get_oll_parity_moves(size: usize) -> Vec<RotationMove> {
    let s = size as i32;
    let mut moves = Vec::new();

    let rw_cw = |m: &mut Vec<RotationMove>| {
        m.push(RotationMove {
            axis: RotationAxis::X,
            index: s - 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        });
        m.push(RotationMove {
            axis: RotationAxis::X,
            index: s - 2,
            direction: Direction::Clockwise,
            add_to_history: true,
        });
    };
    let rw_ccw = |m: &mut Vec<RotationMove>| {
        m.push(RotationMove {
            axis: RotationAxis::X,
            index: s - 1,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        });
        m.push(RotationMove {
            axis: RotationAxis::X,
            index: s - 2,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        });
    };
    let lw_cw = |m: &mut Vec<RotationMove>| {
        m.push(RotationMove {
            axis: RotationAxis::X,
            index: 0,
            direction: Direction::Clockwise,
            add_to_history: true,
        });
        m.push(RotationMove {
            axis: RotationAxis::X,
            index: 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        });
    };
    let lw_ccw = |m: &mut Vec<RotationMove>| {
        m.push(RotationMove {
            axis: RotationAxis::X,
            index: 0,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        });
        m.push(RotationMove {
            axis: RotationAxis::X,
            index: 1,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        });
    };

    let b2 = |m: &mut Vec<RotationMove>| {
        let mv = RotationMove {
            axis: RotationAxis::Z,
            index: 0,
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
    let f2 = |m: &mut Vec<RotationMove>| {
        let mv = RotationMove {
            axis: RotationAxis::Z,
            index: s - 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        };
        m.push(mv);
        m.push(mv);
    };

    // Rw2
    rw_cw(&mut moves);
    rw_cw(&mut moves);
    // B2
    b2(&mut moves);
    // U2
    u2(&mut moves);
    // Lw
    lw_cw(&mut moves);
    // U2
    u2(&mut moves);
    // Rw'
    rw_ccw(&mut moves);
    // U2
    u2(&mut moves);
    // Rw
    rw_cw(&mut moves);
    // U2
    u2(&mut moves);
    // F2
    f2(&mut moves);
    // Rw
    rw_cw(&mut moves);
    // F2
    f2(&mut moves);
    // Lw'
    lw_ccw(&mut moves);
    // B2
    b2(&mut moves);
    // Rw2
    rw_cw(&mut moves);
    rw_cw(&mut moves);

    moves
}

/// Returns the sequence of moves to fix PLL Parity (swapped composite edges)
/// Formula: r2 U2 r2 Uw2 r2 uw2
pub fn get_pll_parity_moves(size: usize) -> Vec<RotationMove> {
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
    // uw2
    uw2(&mut moves);

    moves
}

/// Checks if a 3x3 state string is mathematically solvable
pub fn is_solvable_3x3(state_str: &str) -> bool {
    if let Ok(face_cube) = FaceCube::try_from(state_str) {
        CubieCube::try_from(&face_cube).is_ok()
    } else {
        false
    }
}
