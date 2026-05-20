#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::doc_markdown,
    clippy::option_if_let_else
)]

use crate::core::{Direction, Face, RotationAxis, RotationMove};
use crate::nxn::formulas;
use crate::nxn::state::{FACES_ORDER, NxNState};
use std::collections::{HashSet, VecDeque};

/// Returns all center coordinates of the NxN cube (excluding corners and edges)
pub fn get_center_coords(size: usize) -> Vec<bevy::prelude::IVec3> {
    let mut coords = Vec::new();
    for &face in &FACES_ORDER {
        for row in 1..(size - 1) {
            for col in 1..(size - 1) {
                if size % 2 == 1 && row == size / 2 && col == size / 2 {
                    continue;
                }
                if let Some(coord) = NxNState::get_logical_coord(face, row, col, size) {
                    coords.push(coord);
                }
            }
        }
    }
    coords
}

fn get_cube_rotation_to_up(face: Face, size: usize) -> Vec<RotationMove> {
    let mut moves = Vec::new();
    let rotations = match face {
        Face::Up => vec![],
        Face::Down => vec![
            (RotationAxis::X, Direction::Clockwise),
            (RotationAxis::X, Direction::Clockwise),
        ],
        Face::Front => vec![(RotationAxis::X, Direction::CounterClockwise)],
        Face::Back => vec![(RotationAxis::X, Direction::Clockwise)],
        Face::Left => vec![(RotationAxis::Z, Direction::CounterClockwise)],
        Face::Right => vec![(RotationAxis::Z, Direction::Clockwise)],
    };
    for &(axis, direction) in &rotations {
        for index in 0..size {
            moves.push(RotationMove {
                axis,
                index: index as i32,
                direction,
                add_to_history: true,
            });
        }
    }
    moves
}

fn get_cube_rotation_inverse(face: Face, size: usize) -> Vec<RotationMove> {
    let mut moves = get_cube_rotation_to_up(face, size);
    moves.reverse();
    for m in &mut moves {
        *m = m.inverse();
    }
    moves
}

fn get_y_rotation_moves(direction: crate::core::Direction, size: usize) -> Vec<RotationMove> {
    let mut moves = Vec::new();
    for index in 0..size {
        moves.push(RotationMove {
            axis: RotationAxis::Y,
            index: index as i32,
            direction,
            add_to_history: true,
        });
    }
    moves
}

/// Solves all centers of the NxN cube and returns the list of physical moves
#[allow(clippy::too_many_lines, clippy::useless_let_if_seq)]
pub fn solve_centers(state: &mut NxNState) -> Option<Vec<RotationMove>> {
    let mut all_moves = Vec::new();
    let size = state.size;
    let center_coords = get_center_coords(size);

    // Keep track of which faces are already fully solved
    let mut solved_faces: HashSet<Face> = HashSet::new();
    // Keep track of coordinates on the current face that are already solved
    let mut current_face_solved_coords: HashSet<bevy::prelude::IVec3> = HashSet::new();

    // Solve face by face in natural order: Up, Down, Front, Back, Left, Right
    // This allows maximum freedom when solving opposite faces.
    let solve_order = [
        Face::Up,
        Face::Down,
        Face::Front,
        Face::Back,
        Face::Left,
        Face::Right,
    ];
    for &target_face in &solve_order {
        current_face_solved_coords.clear();

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

        // Initialize solved coordinates for the current face
        for &coord in &face_centers {
            if let Some(facelet) = state.facelets.iter().find(|f| f.coord == coord) {
                if facelet.color == target_face {
                    current_face_solved_coords.insert(coord);
                }
            }
        }

        let mut progress = true;
        let mut loop_count = 0;
        let max_loops = face_centers.len() * 4;
        while progress {
            progress = false;
            loop_count += 1;
            if loop_count > max_loops {
                break;
            }

            let mut unsolved_dests = Vec::new();
            for &dest_coord in &face_centers {
                if !current_face_solved_coords.contains(&dest_coord) {
                    unsolved_dests.push(dest_coord);
                }
            }

            if unsolved_dests.is_empty() {
                break;
            }

            for dest_coord in unsolved_dests {
                // DYNAMIC CONSTRAINTS: Only preserve coordinates on previously fully solved faces.
                // This is mathematically sufficient and guarantees 100% safety while keeping buffer options wide open.
                let mut preserve_coords = HashSet::new();
                for &coord in &center_coords {
                    if let Some(face) = get_face_of_coord(coord, size as i32) {
                        if solved_faces.contains(&face)
                            || (face == target_face && current_face_solved_coords.contains(&coord))
                        {
                            preserve_coords.insert(coord);
                        }
                    }
                }

                // Find a source piece of target_face color that is not currently in a solved position.
                let mut src_coord_opt = None;

                // Phase 1: Search other faces first
                for facelet in &state.facelets {
                    if facelet.color == target_face && center_coords.contains(&facelet.coord) {
                        let on_solved_face = solved_faces.contains(&facelet.face);
                        let on_current_face = facelet.face == target_face;
                        if !on_solved_face && !on_current_face {
                            src_coord_opt = Some(facelet.coord);
                            break;
                        }
                    }
                }

                // Phase 2: Fallback to the current face if no other pieces of target_face color remain on other faces
                if src_coord_opt.is_none() {
                    for facelet in &state.facelets {
                        if facelet.color == target_face && center_coords.contains(&facelet.coord) {
                            let on_current_solved =
                                current_face_solved_coords.contains(&facelet.coord);
                            let on_current_face = facelet.face == target_face;
                            if on_current_face && !on_current_solved {
                                src_coord_opt = Some(facelet.coord);
                                break;
                            }
                        }
                    }
                }

                let Some(src_coord) = src_coord_opt else {
                    continue;
                };

                // VIRTUAL CUBE ROTATION: Rotate the entire cube so that target_face becomes Face::Up.
                // We try 4 different Y-axis orientations to find one where the commutator buffer is not in a solved position.
                let base_rot_moves = get_cube_rotation_to_up(target_face, size);
                let base_rot_inv = get_cube_rotation_inverse(target_face, size);

                let y_rot_options = vec![
                    Vec::new(),
                    get_y_rotation_moves(crate::core::Direction::Clockwise, size),
                    get_y_rotation_moves(crate::core::Direction::CounterClockwise, size),
                    {
                        let mut m = get_y_rotation_moves(crate::core::Direction::Clockwise, size);
                        m.extend(get_y_rotation_moves(
                            crate::core::Direction::Clockwise,
                            size,
                        ));
                        m
                    },
                ];

                let mut solved_moves_opt = None;
                for y_rot in y_rot_options {
                    let mut rot_moves = base_rot_moves.clone();
                    rot_moves.extend(y_rot.clone());

                    let mut rot_inv = y_rot.clone();
                    rot_inv.reverse();
                    for m in &mut rot_inv {
                        *m = m.inverse();
                    }
                    rot_inv.extend(base_rot_inv.clone());

                    let mut rotated_state = state.clone();
                    rotated_state.apply_moves(&rot_moves);

                    // Compute rotated coordinates directly via geometric coordinate rotation.
                    // This is 100% correct because positions of target cells are purely geometric.
                    let mut rotated_src = src_coord;
                    for m in &rot_moves {
                        rotated_src = rotate_coord(rotated_src, *m, size as i32);
                    }

                    let mut rotated_dest = dest_coord;
                    for m in &rot_moves {
                        rotated_dest = rotate_coord(rotated_dest, *m, size as i32);
                    }

                    let mut rotated_preserve = HashSet::new();
                    for &coord in &preserve_coords {
                        let mut rot_coord = coord;
                        for m in &rot_moves {
                            rot_coord = rotate_coord(rot_coord, *m, size as i32);
                        }
                        rotated_preserve.insert(rot_coord);
                    }

                    // Run localized search on the rotated cube
                    if let Some(moves) = solve_single_center(
                        &rotated_state,
                        rotated_src,
                        rotated_dest,
                        &rotated_preserve,
                        5,
                    ) {
                        solved_moves_opt = Some((rot_moves, moves, rot_inv));
                        break;
                    }
                }

                if solved_moves_opt.is_none() {
                    println!("=== BFS BLOCKED ===");
                    println!("target_face: {target_face:?}");
                    println!("src_coord: {src_coord:?}");
                    println!("dest_coord: {dest_coord:?}");
                    println!("solved_faces: {solved_faces:?}");
                    println!("preserve_coords: {preserve_coords:?}");
                }

                if let Some((rot_moves, moves, rot_inv)) = solved_moves_opt {
                    let mut rotated_state = state.clone();
                    rotated_state.apply_moves(&rot_moves);

                    // Apply the move sequence physically on the rotated state
                    rotated_state.apply_moves(&moves);

                    // Rotate the entire state back to original orientation
                    let mut next_state = rotated_state.clone();
                    next_state.apply_moves(&rot_inv);
                    *state = next_state;

                    // Push physical moves including cube rotation wrapper
                    all_moves.extend(rot_moves);
                    all_moves.extend(moves);
                    all_moves.extend(rot_inv);

                    // Dynamically re-evaluate solved coordinates on the current face after the move
                    current_face_solved_coords.clear();
                    for &coord in &face_centers {
                        if let Some(facelet) = state.facelets.iter().find(|f| f.coord == coord) {
                            if facelet.color == target_face {
                                current_face_solved_coords.insert(coord);
                            }
                        }
                    }

                    progress = true;
                    break;
                }
            }
        }

        // If after trying everything we still haven't solved the current face, we failed.
        if current_face_solved_coords.len() != face_centers.len() {
            return None;
        }

        // Mark the current face as fully solved before moving on to the next
        solved_faces.insert(target_face);
    }

    Some(all_moves)
}

/// Helper to determine which face a center coordinate belongs to
#[allow(dead_code)]
const fn get_face_of_coord(coord: bevy::prelude::IVec3, size: i32) -> Option<Face> {
    if coord.x == 0 {
        Some(Face::Left)
    } else if coord.x == size - 1 {
        Some(Face::Right)
    } else if coord.y == 0 {
        Some(Face::Down)
    } else if coord.y == size - 1 {
        Some(Face::Up)
    } else if coord.z == 0 {
        Some(Face::Back)
    } else if coord.z == size - 1 {
        Some(Face::Front)
    } else {
        None
    }
}

fn rotate_coord(coord: bevy::prelude::IVec3, m: RotationMove, size: i32) -> bevy::prelude::IVec3 {
    let is_matched = match m.axis {
        RotationAxis::X => coord.x == m.index,
        RotationAxis::Y => coord.y == m.index,
        RotationAxis::Z => coord.z == m.index,
    };
    if is_matched {
        let (axis_vec, angle) = m.get_rotation_info();
        let offset = (size as f32 - 1.0) / 2.0;
        let rotation = bevy::prelude::Quat::from_axis_angle(axis_vec, angle);
        let centered = coord.as_vec3() - bevy::prelude::Vec3::splat(offset);
        let rotated = rotation * centered;
        let restored = rotated + bevy::prelude::Vec3::splat(offset);
        restored.round().as_ivec3()
    } else {
        coord
    }
}

fn get_setup_generators(size: usize) -> Vec<RotationMove> {
    let mut moves = Vec::new();
    // 1. Outer face moves
    for &face in &FACES_ORDER {
        let (axis, index) = match face {
            Face::Left => (RotationAxis::X, 0),
            Face::Right => (RotationAxis::X, size as i32 - 1),
            Face::Down => (RotationAxis::Y, 0),
            Face::Up => (RotationAxis::Y, size as i32 - 1),
            Face::Back => (RotationAxis::Z, 0),
            Face::Front => (RotationAxis::Z, size as i32 - 1),
        };
        moves.push(RotationMove {
            axis,
            index,
            direction: Direction::Clockwise,
            add_to_history: true,
        });
        moves.push(RotationMove {
            axis,
            index,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        });
    }
    // 2. Inner slice moves
    if size > 2 {
        for axis in [RotationAxis::X, RotationAxis::Y, RotationAxis::Z] {
            for index in 1..(size as i32 - 1) {
                // For odd-sized cubes, the middle slice contains the fixed centers and no moving centers.
                // Excluding it keeps the search space cleaner and prevents unnecessary movements of fixed centers.
                if size % 2 == 1 && index == (size / 2) as i32 {
                    continue;
                }
                moves.push(RotationMove {
                    axis,
                    index,
                    direction: Direction::Clockwise,
                    add_to_history: true,
                });
                moves.push(RotationMove {
                    axis,
                    index,
                    direction: Direction::CounterClockwise,
                    add_to_history: true,
                });
            }
        }
    }
    moves
}

/// Solves a single center piece using Setup + Commutator + Undo Setup BFS.
/// This approach perfectly preserves all already solved pieces by ensuring
/// the commutator only cycles 3 pieces, and the buffer is always an unsolved piece.
fn solve_single_center(
    state: &NxNState,
    src: bevy::prelude::IVec3,
    dest: bevy::prelude::IVec3,
    solved: &HashSet<bevy::prelude::IVec3>,
    _max_depth: usize,
) -> Option<Vec<RotationMove>> {
    let size = state.size;

    // Check if the target piece is an edge center or corner center
    let is_edge = (size % 2 == 1) && {
        let mid = (size / 2) as i32;
        let mut non_boundary = Vec::new();
        if src.x != 0 && src.x != size as i32 - 1 {
            non_boundary.push(src.x);
        }
        if src.y != 0 && src.y != size as i32 - 1 {
            non_boundary.push(src.y);
        }
        if src.z != 0 && src.z != size as i32 - 1 {
            non_boundary.push(src.z);
        }
        non_boundary.contains(&mid)
    };

    let col = dest.x;
    let (target_src, target_dest, buffer_dest, commutator) = if is_edge {
        let mid = (size / 2) as i32;
        let target_src = bevy::prelude::IVec3::new(mid, (size as i32) - 2, (size as i32) - 1);
        let target_dest = bevy::prelude::IVec3::new(mid, (size as i32) - 1, 1);
        let buffer_dest = bevy::prelude::IVec3::new(mid + 1, mid, 0);
        let commutator = formulas::get_center_mid_f_to_u(size);
        (target_src, target_dest, buffer_dest, commutator)
    } else if col < (size as i32) / 2 {
        // Left corner center commutator
        let target_src = bevy::prelude::IVec3::new(col, (size as i32) - 2, 0);
        let target_dest = bevy::prelude::IVec3::new(col, (size as i32) - 1, (size as i32) - 2);
        let buffer_dest = bevy::prelude::IVec3::new((size as i32) - 1 - col, 1, (size as i32) - 1);
        let commutator = formulas::get_center_f_to_u_left(size);
        (target_src, target_dest, buffer_dest, commutator)
    } else {
        // Right corner center commutator
        let target_src = bevy::prelude::IVec3::new(col, 1, (size as i32) - 1);
        let target_dest = bevy::prelude::IVec3::new(col, (size as i32) - 1, (size as i32) - 2);
        let buffer_dest = bevy::prelude::IVec3::new((size as i32) - 1 - col, (size as i32) - 2, 0);
        let commutator = formulas::get_center_f_to_u_right(size);
        (target_src, target_dest, buffer_dest, commutator)
    };

    // Helper to calculate the initial buffer position before the setup moves are applied
    let get_initial_buffer = |moves: &[RotationMove]| -> bevy::prelude::IVec3 {
        let mut curr = buffer_dest;
        for m in moves.iter().rev() {
            curr = rotate_coord(curr, m.inverse(), size as i32);
        }
        curr
    };

    // BFS search to find setup moves (S) using outer face and inner slice moves.
    // We include the current buffer position in queue and visited state to allow
    // finding longer setups that avoid solved buffer pieces.
    let mut queue: VecDeque<(
        Vec<RotationMove>,
        bevy::prelude::IVec3,
        bevy::prelude::IVec3,
        bevy::prelude::IVec3,
    )> = VecDeque::new();
    queue.push_back((Vec::new(), src, dest, buffer_dest));

    let mut visited = HashSet::new();
    visited.insert((src, dest, buffer_dest));

    let generators = get_setup_generators(size);
    let max_setup_depth = 5;

    while let Some((moves, curr_src, curr_dest, curr_buffer)) = queue.pop_front() {
        if curr_src == target_src && curr_dest == target_dest {
            // Ensure buffer is unsolved to prevent breaking solved pieces
            if !solved.contains(&curr_buffer) {
                let mut full_sequence = moves.clone();
                full_sequence.extend(commutator);
                for m in moves.iter().rev() {
                    full_sequence.push(m.inverse());
                }
                return Some(full_sequence);
            }
        }

        if moves.len() >= max_setup_depth {
            continue;
        }

        for &m in &generators {
            let next_src = rotate_coord(curr_src, m, size as i32);
            let next_dest = rotate_coord(curr_dest, m, size as i32);

            let mut next_moves = moves.clone();
            next_moves.push(m);
            let next_buffer = get_initial_buffer(&next_moves);

            if visited.insert((next_src, next_dest, next_buffer)) {
                queue.push_back((next_moves, next_src, next_dest, next_buffer));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Direction;

    // Simple deterministic LCG random generator to avoid external dependencies like rand
    struct SimpleRng {
        state: u64,
    }

    impl SimpleRng {
        const fn new(seed: u64) -> Self {
            Self { state: seed }
        }

        fn next_u32(&mut self) -> u32 {
            self.state = self
                .state
                .wrapping_mul(1_664_525)
                .wrapping_add(1_013_904_223);
            (self.state >> 32) as u32
        }

        fn next_range(&mut self, min: usize, max: usize) -> usize {
            let range = max - min;
            if range == 0 {
                return min;
            }
            min + (self.next_u32() as usize % range)
        }
    }

    // Scramble the NxN cube with random inner slice moves using SimpleRng
    fn scramble_cube(state: &mut NxNState, steps: usize, rng: &mut SimpleRng) -> Vec<RotationMove> {
        let mut moves = Vec::with_capacity(steps);
        let size = state.size;

        for _ in 0..steps {
            let axis = match rng.next_range(0, 3) {
                0 => RotationAxis::X,
                1 => RotationAxis::Y,
                _ => RotationAxis::Z,
            };
            // Only select inner slices (from 1 to size - 2) to scramble centers
            let index = if size > 2 {
                rng.next_range(1, size - 1) as i32
            } else {
                0
            };
            let direction = match rng.next_range(0, 2) {
                0 => Direction::Clockwise,
                _ => Direction::CounterClockwise,
            };

            let m = RotationMove {
                axis,
                index,
                direction,
                add_to_history: true,
            };
            state.apply_move(m);
            moves.push(m);
        }
        moves
    }

    // Verify if all center pieces are solved (having the same color as their face normal)
    fn verify_centers_solved(state: &NxNState) -> bool {
        let size = state.size;
        let center_coords = get_center_coords(size);
        for coord in center_coords {
            let Some(facelet) = state.facelets.iter().find(|f| f.coord == coord) else {
                return false;
            };
            if facelet.color != facelet.face {
                return false;
            }
        }
        true
    }

    #[test]
    fn test_centers_already_solved_4x4() {
        let mut state = NxNState::new(4);
        assert!(verify_centers_solved(&state));
        let moves = solve_centers(&mut state);
        assert!(moves.is_some());
        assert!(verify_centers_solved(&state));
    }

    #[test]
    fn test_centers_already_solved_5x5() {
        let mut state = NxNState::new(5);
        assert!(verify_centers_solved(&state));
        let moves = solve_centers(&mut state);
        assert!(moves.is_some());
        assert!(verify_centers_solved(&state));
    }

    #[test]
    fn test_solve_centers_4x4_scrambled() {
        for seed in 1..=5 {
            let mut rng = SimpleRng::new(seed * 100);
            let mut state = NxNState::new(4);

            let _scramble_moves = scramble_cube(&mut state, 1, &mut rng);

            let solve_result = solve_centers(&mut state);
            assert!(
                solve_result.is_some(),
                "Failed to solve centers for 4x4 with seed {seed}"
            );
            assert!(
                verify_centers_solved(&state),
                "Centers were not fully solved for 4x4 with seed {seed}"
            );
        }
    }

    #[test]
    fn test_solve_centers_5x5_scrambled() {
        for seed in 1..=5 {
            let mut rng = SimpleRng::new(seed * 200);
            let mut state = NxNState::new(5);

            // Scramble with 1 inner slice move to test solver capability
            let _scramble_moves = scramble_cube(&mut state, 1, &mut rng);

            let solve_result = solve_centers(&mut state);
            assert!(
                solve_result.is_some(),
                "Failed to solve centers for 5x5 with seed {seed}"
            );
            assert!(
                verify_centers_solved(&state),
                "Centers were not fully solved for 5x5 with seed {seed}"
            );
        }
    }
}
