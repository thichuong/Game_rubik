#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::missing_const_for_fn
)]

use crate::core::{Direction, RotationAxis, RotationMove};

/// Helper to get the 'r' inner slice rotation move (adjacent to Right face)
pub const fn get_r_move(size: usize, direction: Direction) -> RotationMove {
    RotationMove {
        axis: RotationAxis::X,
        index: (size as i32) - 2,
        direction,
        add_to_history: true,
    }
}

/// Helper to get the 'l' inner slice rotation move (adjacent to Left face)
pub const fn get_l_move(_size: usize, direction: Direction) -> RotationMove {
    RotationMove {
        axis: RotationAxis::X,
        index: 1,
        direction,
        add_to_history: true,
    }
}

/// Helper to get the 'm' middle slice rotation move for odd-sized cubes
pub const fn get_m_move(size: usize, direction: Direction) -> RotationMove {
    RotationMove {
        axis: RotationAxis::X,
        index: (size as i32) / 2,
        direction,
        add_to_history: true,
    }
}

/// Helper to get the 'U' outer face rotation move
pub const fn get_u_move(size: usize, direction: Direction) -> RotationMove {
    RotationMove {
        axis: RotationAxis::Y,
        index: (size as i32) - 1,
        direction,
        add_to_history: true,
    }
}

/// Upgraded BFS Commutator actions (B)
/// Target Piece: Corner center (4x4 or 5x5 corner centers)
/// Formula: r U l' U' r' U l U'
/// Function: Shoots a center piece from Front-Top-Right to Up-Bottom-Right
pub fn get_center_f_to_u_right(size: usize) -> Vec<RotationMove> {
    vec![
        get_r_move(size, Direction::Clockwise),
        get_u_move(size, Direction::Clockwise),
        get_l_move(size, Direction::CounterClockwise),
        get_u_move(size, Direction::CounterClockwise),
        get_r_move(size, Direction::CounterClockwise),
        get_u_move(size, Direction::Clockwise),
        get_l_move(size, Direction::Clockwise),
        get_u_move(size, Direction::CounterClockwise),
    ]
}

/// Target Piece: Corner center (4x4 or 5x5 corner centers)
/// Formula: r2 U l' U' r2' U l U'
/// Function: Shoots a center piece from Down-Top-Right straight up to Up-Bottom-Right
pub fn get_center_d_to_u_right(size: usize) -> Vec<RotationMove> {
    vec![
        get_r_move(size, Direction::Clockwise),
        get_r_move(size, Direction::Clockwise),
        get_u_move(size, Direction::Clockwise),
        get_l_move(size, Direction::CounterClockwise),
        get_u_move(size, Direction::CounterClockwise),
        get_r_move(size, Direction::CounterClockwise),
        get_r_move(size, Direction::CounterClockwise),
        get_u_move(size, Direction::Clockwise),
        get_l_move(size, Direction::Clockwise),
        get_u_move(size, Direction::CounterClockwise),
    ]
}

/// Target Piece: Edge center (Odd size middle axis 'm' center)
/// Formula: m U r' U' m' U r U'
/// Function: Shoots a center piece from Front-Top-Middle to Up-Bottom-Middle
pub fn get_center_mid_f_to_u(size: usize) -> Vec<RotationMove> {
    vec![
        get_m_move(size, Direction::Clockwise),
        get_u_move(size, Direction::Clockwise),
        get_r_move(size, Direction::CounterClockwise),
        get_u_move(size, Direction::CounterClockwise),
        get_m_move(size, Direction::CounterClockwise),
        get_u_move(size, Direction::Clockwise),
        get_r_move(size, Direction::Clockwise),
        get_u_move(size, Direction::CounterClockwise),
    ]
}

/// Target Piece: Corner center (4x4 or 5x5 corner centers)
/// Formula: l' U' r U l U' r' U
/// Function: Shoots a center piece from Front-Top-Left to Up-Bottom-Left
pub fn get_center_f_to_u_left(size: usize) -> Vec<RotationMove> {
    vec![
        get_l_move(size, Direction::CounterClockwise),
        get_u_move(size, Direction::CounterClockwise),
        get_r_move(size, Direction::Clockwise),
        get_u_move(size, Direction::Clockwise),
        get_l_move(size, Direction::Clockwise),
        get_u_move(size, Direction::CounterClockwise),
        get_r_move(size, Direction::CounterClockwise),
        get_u_move(size, Direction::Clockwise),
    ]
}

/// Target Piece: L2C Corner center (Right side corner center)
/// Formula: r U2 r' U' r U' r'
/// Function: Swaps a right corner center between Left and Right faces, preserving other faces
pub fn get_center_l2c_right(size: usize) -> Vec<RotationMove> {
    vec![
        get_r_move(size, Direction::Clockwise),
        get_u_move(size, Direction::Clockwise),
        get_u_move(size, Direction::Clockwise),
        get_r_move(size, Direction::CounterClockwise),
        get_u_move(size, Direction::CounterClockwise),
        get_r_move(size, Direction::Clockwise),
        get_u_move(size, Direction::CounterClockwise),
        get_r_move(size, Direction::CounterClockwise),
    ]
}

/// Target Piece: L2C Corner center (Left side corner center)
/// Formula: l' U2 l U l' U l
/// Function: Swaps a left corner center between Left and Right faces, preserving other faces
pub fn get_center_l2c_left(size: usize) -> Vec<RotationMove> {
    vec![
        get_l_move(size, Direction::CounterClockwise),
        get_u_move(size, Direction::Clockwise),
        get_u_move(size, Direction::Clockwise),
        get_l_move(size, Direction::Clockwise),
        get_u_move(size, Direction::Clockwise),
        get_l_move(size, Direction::CounterClockwise),
        get_u_move(size, Direction::Clockwise),
        get_l_move(size, Direction::Clockwise),
    ]
}

/// Target Piece: L2C Mid-edge center (Odd size middle axis 'm' center)
/// Formula: m U2 m' U' m U' m'
/// Function: Swaps a middle edge center between Left and Right faces, preserving other faces
pub fn get_center_l2c_mid(size: usize) -> Vec<RotationMove> {
    vec![
        get_m_move(size, Direction::Clockwise),
        get_u_move(size, Direction::Clockwise),
        get_u_move(size, Direction::Clockwise),
        get_m_move(size, Direction::CounterClockwise),
        get_u_move(size, Direction::CounterClockwise),
        get_m_move(size, Direction::Clockwise),
        get_u_move(size, Direction::CounterClockwise),
        get_m_move(size, Direction::CounterClockwise),
    ]
}

/// Target: Edge Flip Algorithm (EDGE_FLIP_ALGO)
/// Formula: R U R' F R' F' R
/// Function: Flips the orientation of the edge at the Front-Right (FR) position
pub fn get_edge_flip_algo(size: usize) -> Vec<RotationMove> {
    let s = size as i32;
    vec![
        RotationMove {
            axis: RotationAxis::X,
            index: s - 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: s - 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: s - 1,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: s - 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: s - 1,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: s - 1,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: s - 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
    ]
}

/// Target: Standard Edge Pairing (EDGE_PAIR_STANDARD)
/// Formula: u' (R U R' F R' F' R) u
/// Function: Pairs two wings brought to the Front-Right and Front-Left positions at slice_idx
pub fn get_edge_pair_standard(size: usize, slice_idx: i32) -> Vec<RotationMove> {
    let mut moves = vec![RotationMove {
        axis: RotationAxis::Y,
        index: slice_idx,
        direction: Direction::CounterClockwise,
        add_to_history: true,
    }];
    moves.extend(get_edge_flip_algo(size));
    moves.push(RotationMove {
        axis: RotationAxis::Y,
        index: slice_idx,
        direction: Direction::Clockwise,
        add_to_history: true,
    });
    moves
}

/// Target: Last Two Edges Fix (LAST_TWO_EDGES_FIX)
/// Formula: d R F' U R' F d'
/// Function: Resolves parity or correctly pairs the last two remaining edge groups
pub fn get_last_two_edges_fix(size: usize, slice_idx: i32) -> Vec<RotationMove> {
    let s = size as i32;
    vec![
        // d
        RotationMove {
            axis: RotationAxis::Y,
            index: slice_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        // R
        RotationMove {
            axis: RotationAxis::X,
            index: s - 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        // F'
        RotationMove {
            axis: RotationAxis::Z,
            index: s - 1,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        // U
        RotationMove {
            axis: RotationAxis::Y,
            index: s - 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        // R'
        RotationMove {
            axis: RotationAxis::X,
            index: s - 1,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        // F
        RotationMove {
            axis: RotationAxis::Z,
            index: s - 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        // d'
        RotationMove {
            axis: RotationAxis::Y,
            index: slice_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
    ]
}
