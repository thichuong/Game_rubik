#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::doc_markdown,
    clippy::option_if_let_else
)]

use crate::core::{Direction, Face, RotationAxis, RotationMove};
use crate::nxn::state::{FACES_ORDER, NxNState};
use std::collections::{HashSet, VecDeque};

/// Returns all center coordinates of the NxN cube (excluding corners and edges)
pub fn get_center_coords(size: usize) -> Vec<bevy::prelude::IVec3> {
    let mut coords = Vec::new();
    for &face in &FACES_ORDER {
        for row in 1..(size - 1) {
            for col in 1..(size - 1) {
                if let Some(coord) = NxNState::get_logical_coord(face, row, col, size) {
                    coords.push(coord);
                }
            }
        }
    }
    coords
}

/// Solves all centers of the NxN cube and returns the list of physical moves
pub fn solve_centers(state: &mut NxNState) -> Option<Vec<RotationMove>> {
    let mut all_moves = Vec::new();
    let size = state.size;
    let center_coords = get_center_coords(size);

    // Keep track of coordinates that are already solved (correct color on correct face)
    let mut solved_coords: HashSet<bevy::prelude::IVec3> = HashSet::new();

    // Initialize solved coordinates
    for &coord in &center_coords {
        if let Some(facelet) = state.facelets.iter().find(|f| f.coord == coord) {
            if facelet.color == facelet.face {
                solved_coords.insert(coord);
            }
        }
    }

    // Solve face by face in standard order: Up, Down, Front, Back, Left, Right
    for &target_face in &FACES_ORDER {
        let face_centers: Vec<bevy::prelude::IVec3> = center_coords
            .iter()
            .copied()
            .filter(|&coord| {
                if let Some(facelet) = state.facelets.iter().find(|f| f.coord == coord) {
                    facelet.face == target_face
                } else {
                    false
                }
            })
            .collect();

        for &dest_coord in &face_centers {
            // Check if this specific destination is already solved with correct color
            if let Some(facelet) = state.facelets.iter().find(|f| f.coord == dest_coord) {
                if facelet.color == target_face {
                    solved_coords.insert(dest_coord);
                    continue;
                }
            }

            // Find a piece with color target_face that is currently not solved
            let mut src_coord_opt = None;
            for facelet in &state.facelets {
                if facelet.color == target_face
                    && !solved_coords.contains(&facelet.coord)
                    && center_coords.contains(&facelet.coord)
                {
                    src_coord_opt = Some(facelet.coord);
                    break;
                }
            }

            let Some(src_coord) = src_coord_opt else {
                // If no source piece of this color is found in centers, there's a state error or it's already solved
                continue;
            };

            // Run localized BFS to move src_coord to dest_coord without disrupting solved_coords
            if let Some(moves) = solve_single_center(state, src_coord, dest_coord, &solved_coords) {
                state.apply_moves(&moves);
                all_moves.extend(moves);
                solved_coords.insert(dest_coord);
            } else {
                // If single center BFS failed, we can try to scramble briefly or return None
                return None;
            }
        }
    }

    Some(all_moves)
}

/// Find a sequence of moves to solve a single center piece
fn solve_single_center(
    state: &NxNState,
    src: bevy::prelude::IVec3,
    dest: bevy::prelude::IVec3,
    solved: &HashSet<bevy::prelude::IVec3>,
) -> Option<Vec<RotationMove>> {
    let size = state.size;

    // Define permissible generator moves for center solving to keep search space small
    let mut generators = Vec::new();

    // 1. Standard face rotations (U, D, R, L, F, B)
    for &face in &FACES_ORDER {
        let (axis, index) = match face {
            Face::Left => (RotationAxis::X, 0),
            Face::Right => (RotationAxis::X, size as i32 - 1),
            Face::Down => (RotationAxis::Y, 0),
            Face::Up => (RotationAxis::Y, size as i32 - 1),
            Face::Back => (RotationAxis::Z, 0),
            Face::Front => (RotationAxis::Z, size as i32 - 1),
        };
        generators.push(RotationMove {
            axis,
            index,
            direction: Direction::Clockwise,
            add_to_history: true,
        });
        generators.push(RotationMove {
            axis,
            index,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        });
    }

    // 2. Inner slice moves (index from 1 to size-2)
    for idx in 1..(size - 1) {
        for &axis in &[RotationAxis::X, RotationAxis::Y, RotationAxis::Z] {
            generators.push(RotationMove {
                axis,
                index: idx as i32,
                direction: Direction::Clockwise,
                add_to_history: true,
            });
            generators.push(RotationMove {
                axis,
                index: idx as i32,
                direction: Direction::CounterClockwise,
                add_to_history: true,
            });
        }
    }

    // BFS Search queue: stores (current_state, moves_taken, last_piece_position)
    let mut queue = VecDeque::new();
    queue.push_back((state.clone(), Vec::new(), src));

    let mut visited = HashSet::new();
    visited.insert(state.to_string_rep());

    // Set maximum depth of 4 to keep search fast and prevent explosion
    let max_depth = 4;

    while let Some((curr_state, moves, curr_pos)) = queue.pop_front() {
        if curr_pos == dest {
            return Some(moves);
        }

        if moves.len() >= max_depth {
            continue;
        }

        for &m in &generators {
            // Apply move to state
            let mut next_state = curr_state.clone();
            next_state.apply_move(m);

            // Check if this move disrupted any solved coordinates
            let mut disrupted = false;
            for &coord in solved {
                let prev_color = curr_state
                    .facelets
                    .iter()
                    .find(|f| f.coord == coord)
                    .map(|f| f.color);
                let next_color = next_state
                    .facelets
                    .iter()
                    .find(|f| f.coord == coord)
                    .map(|f| f.color);
                if prev_color != next_color {
                    disrupted = true;
                    break;
                }
            }

            if disrupted {
                continue;
            }

            // Find new position of the source piece
            let mut next_pos = curr_pos;
            let (axis_vec, angle) = m.get_rotation_info();
            let size_i32 = size as i32;
            let is_matched = match m.axis {
                RotationAxis::X => curr_pos.x == m.index,
                RotationAxis::Y => curr_pos.y == m.index,
                RotationAxis::Z => curr_pos.z == m.index,
            };

            if is_matched {
                let offset = (size_i32 as f32 - 1.0) / 2.0;
                let rotation = bevy::prelude::Quat::from_axis_angle(axis_vec, angle);
                let centered = curr_pos.as_vec3() - bevy::prelude::Vec3::splat(offset);
                let rotated = rotation * centered;
                let restored = rotated + bevy::prelude::Vec3::splat(offset);
                next_pos = restored.round().as_ivec3();
            }

            let state_rep = next_state.to_string_rep();
            if visited.insert(state_rep) {
                let mut next_moves = moves.clone();
                next_moves.push(m);
                queue.push_back((next_state, next_moves, next_pos));
            }
        }
    }

    None
}
