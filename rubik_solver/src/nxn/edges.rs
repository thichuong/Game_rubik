#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::doc_markdown,
    clippy::too_many_lines,
    clippy::similar_names,
    clippy::redundant_else,
    clippy::uninlined_format_args
)]

use crate::core::{Direction, Face, RotationAxis, RotationMove};
use crate::nxn::state::{FACES_ORDER, NxNState};
use std::collections::{HashSet, VecDeque};

pub const COMPOSITE_EDGES: [(Face, Face); 12] = [
    (Face::Up, Face::Back),
    (Face::Up, Face::Right),
    (Face::Up, Face::Front),
    (Face::Up, Face::Left),
    (Face::Down, Face::Back),
    (Face::Down, Face::Right),
    (Face::Down, Face::Front),
    (Face::Down, Face::Left),
    (Face::Front, Face::Right),
    (Face::Front, Face::Left),
    (Face::Back, Face::Right),
    (Face::Back, Face::Left),
];

/// Get the wings coordinates for a composite edge
pub fn get_edge_wings(f1: Face, f2: Face, size: usize) -> Vec<bevy::prelude::IVec3> {
    let mut wings = Vec::new();
    let s = size as i32;

    let (a, b) = if (f1 as usize) < (f2 as usize) {
        (f1, f2)
    } else {
        (f2, f1)
    };

    for idx in 1..(s - 1) {
        match (a, b) {
            (Face::Up, Face::Back) => wings.push(bevy::prelude::IVec3::new(idx, s - 1, 0)),
            (Face::Up, Face::Right) => wings.push(bevy::prelude::IVec3::new(s - 1, s - 1, idx)),
            (Face::Up, Face::Front) => wings.push(bevy::prelude::IVec3::new(idx, s - 1, s - 1)),
            (Face::Up, Face::Left) => wings.push(bevy::prelude::IVec3::new(0, s - 1, idx)),

            (Face::Down, Face::Back) => wings.push(bevy::prelude::IVec3::new(idx, 0, 0)),
            (Face::Down, Face::Right) => wings.push(bevy::prelude::IVec3::new(s - 1, 0, idx)),
            (Face::Down, Face::Front) => wings.push(bevy::prelude::IVec3::new(idx, 0, s - 1)),
            (Face::Down, Face::Left) => wings.push(bevy::prelude::IVec3::new(0, 0, idx)),

            (Face::Front, Face::Right) => wings.push(bevy::prelude::IVec3::new(s - 1, idx, s - 1)),
            (Face::Front, Face::Left) => wings.push(bevy::prelude::IVec3::new(0, idx, s - 1)),
            (Face::Back, Face::Right) => wings.push(bevy::prelude::IVec3::new(s - 1, idx, 0)),
            (Face::Back, Face::Left) => wings.push(bevy::prelude::IVec3::new(0, idx, 0)),
            _ => {}
        }
    }
    wings
}

/// Helper to get the 2 colors of a wing piece at coordinate
fn get_wing_colors(
    state: &NxNState,
    coord: bevy::prelude::IVec3,
    f1: Face,
    f2: Face,
) -> Option<(Face, Face)> {
    let c1 = state
        .facelets
        .iter()
        .find(|f| f.coord == coord && f.face == f1)
        .map(|f| f.color)?;
    let c2 = state
        .facelets
        .iter()
        .find(|f| f.coord == coord && f.face == f2)
        .map(|f| f.color)?;
    Some((c1, c2))
}

/// Check if a composite edge is correctly paired (all wings have identical aligned colors)
pub fn is_edge_paired(state: &NxNState, f1: Face, f2: Face) -> bool {
    let wings = get_edge_wings(f1, f2, state.size);
    if wings.is_empty() {
        return true;
    }

    let Some(first_colors) = get_wing_colors(state, wings[0], f1, f2) else {
        return false;
    };

    for &coord in &wings[1..] {
        let Some(colors) = get_wing_colors(state, coord, f1, f2) else {
            return false;
        };
        if colors != first_colors {
            return false;
        }
    }
    true
}

/// Pairs all edges of the NxN cube and returns the list of physical moves
pub fn pair_edges(state: &mut NxNState) -> Option<Vec<RotationMove>> {
    let mut all_moves = Vec::new();
    let size = state.size;

    // Keep track of paired edges
    let mut paired_edges = HashSet::new();
    for &(f1, f2) in &COMPOSITE_EDGES {
        if is_edge_paired(state, f1, f2) {
            paired_edges.insert((f1, f2));
        }
    }

    let mut loop_count = 0;
    let max_loops = 100;

    while paired_edges.len() < 10 && loop_count < max_loops {
        loop_count += 1;
        let mut progress = false;

        'outer_pair: for &(f1, f2) in &COMPOSITE_EDGES {
            if paired_edges.contains(&(f1, f2)) {
                continue;
            }

            let wings = get_edge_wings(f1, f2, size);
            if wings.is_empty() {
                continue;
            }

            let Some(target_colors) = get_wing_colors(state, wings[0], f1, f2) else {
                continue;
            };

            for &dest_coord in &wings[1..] {
                if let Some(colors) = get_wing_colors(state, dest_coord, f1, f2) {
                    if colors == target_colors {
                        continue;
                    }
                }

                // Find an unpaired wing piece with matching colors
                let mut src_coord_opt = None;
                'src_search: for &(sf1, sf2) in &COMPOSITE_EDGES {
                    if paired_edges.contains(&(sf1, sf2)) || (sf1 == f1 && sf2 == f2) {
                        continue;
                    }
                    let swings = get_edge_wings(sf1, sf2, size);
                    for &sc in &swings {
                        if let Some(colors) = get_wing_colors(state, sc, sf1, sf2) {
                            if (colors.0 == target_colors.0 && colors.1 == target_colors.1)
                                || (colors.0 == target_colors.1 && colors.1 == target_colors.0)
                            {
                                src_coord_opt = Some(sc);
                                break 'src_search;
                            }
                        }
                    }
                }

                let Some(src_coord) = src_coord_opt else {
                    continue;
                };

                // Run fast geometry BFS to pair this wing
                if let Some(moves) = solve_single_wing(state, src_coord, dest_coord, size) {
                    state.apply_moves(&moves);
                    all_moves.extend(moves);
                    progress = true;
                    break 'outer_pair;
                } else {
                    return None;
                }
            }
        }

        // Dynamically update paired edges list
        paired_edges.clear();
        for &(f1, f2) in &COMPOSITE_EDGES {
            if is_edge_paired(state, f1, f2) {
                paired_edges.insert((f1, f2));
            }
        }

        if !progress {
            break;
        }
    }

    // Solve Last Two Edges (L2E) if there are exactly 2 unpaired edges left
    let unpaired_edges: Vec<(Face, Face)> = COMPOSITE_EDGES
        .iter()
        .copied()
        .filter(|&edge| !is_edge_paired(state, edge.0, edge.1))
        .collect();

    if unpaired_edges.len() == 2 {
        let edge1 = unpaired_edges[0];
        let edge2 = unpaired_edges[1];

        if let Some(setup_moves) = find_l2e_setup(edge1, edge2, size) {
            state.apply_moves(&setup_moves);
            all_moves.extend(setup_moves.clone());

            let wings1 = get_edge_wings(Face::Front, Face::Left, size);
            let wings2 = get_edge_wings(Face::Front, Face::Right, size);

            for idx in 0..wings1.len() {
                let w1_coord = wings1[idx];
                let w2_coord = wings2[idx];
                let c1 = get_wing_colors(state, w1_coord, Face::Front, Face::Left);
                let c2 = get_wing_colors(state, w2_coord, Face::Front, Face::Right);

                if c1 != c2 {
                    let slice_idx = w1_coord.y;
                    let l2e_moves = get_l2e_formula_moves(size, slice_idx);
                    state.apply_moves(&l2e_moves);
                    all_moves.extend(l2e_moves);
                }
            }

            // Undo L2E setup
            let mut undo_setup = setup_moves;
            undo_setup.reverse();
            for m in &mut undo_setup {
                *m = m.inverse();
            }
            state.apply_moves(&undo_setup);
            all_moves.extend(undo_setup);
        }
    }

    // Verify all edges are correctly paired at the end
    for &(f1, f2) in &COMPOSITE_EDGES {
        if !is_edge_paired(state, f1, f2) {
            return None;
        }
    }

    Some(all_moves)
}

/// Rotation helper for coordinate tracking in BFS
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

/// Generate 12 standard outer face moves U, D, R, L, F, B
fn get_outer_generators(size: usize) -> Vec<RotationMove> {
    let mut moves = Vec::new();
    let s = size as i32;
    for &face in &FACES_ORDER {
        let (axis, index) = match face {
            Face::Left => (RotationAxis::X, 0),
            Face::Right => (RotationAxis::X, s - 1),
            Face::Down => (RotationAxis::Y, 0),
            Face::Up => (RotationAxis::Y, s - 1),
            Face::Back => (RotationAxis::Z, 0),
            Face::Front => (RotationAxis::Z, s - 1),
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
    moves
}

/// Standard Flipping Macro: R U R' F R' F' R
fn get_flipping_macro(size: usize) -> Vec<RotationMove> {
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

/// BFS to find setup moves for a single wing pairing using only outer face moves
fn solve_single_wing(
    state: &NxNState,
    src: bevy::prelude::IVec3,
    dest: bevy::prelude::IVec3,
    size: usize,
) -> Option<Vec<RotationMove>> {
    let s = size as i32;
    let generators = get_outer_generators(size);

    let mut queue = VecDeque::new();
    queue.push_back((Vec::new(), src, dest));

    let mut visited = HashSet::new();
    visited.insert((src, dest));

    let max_setup_depth = 4;

    while let Some((moves, curr_src, curr_dest)) = queue.pop_front() {
        // Target: dest at FR (s-1, slice_idx, s-1), src at FL (0, slice_idx, s-1)
        if curr_dest.x == s - 1
            && curr_dest.z == s - 1
            && curr_src.x == 0
            && curr_src.z == s - 1
            && curr_dest.y == curr_src.y
            && curr_dest.y > 0
            && curr_dest.y < s - 1
        {
            let slice_idx = curr_dest.y;
            let y_ref = if slice_idx == 1 { s - 2 } else { 1 };

            let mut temp_state = state.clone();
            temp_state.apply_moves(&moves);

            // Try slice 90 degrees Clockwise to check color alignment
            let slice_move = RotationMove {
                axis: RotationAxis::Y,
                index: slice_idx,
                direction: Direction::Clockwise,
                add_to_history: true,
            };
            let mut test_state = temp_state.clone();
            test_state.apply_move(slice_move);

            if let Some(colors_at_slice) = get_wing_colors(
                &test_state,
                bevy::prelude::IVec3::new(s - 1, slice_idx, s - 1),
                Face::Front,
                Face::Right,
            ) {
                if let Some(colors_ref) = get_wing_colors(
                    &test_state,
                    bevy::prelude::IVec3::new(s - 1, y_ref, s - 1),
                    Face::Front,
                    Face::Right,
                ) {
                    if colors_at_slice == colors_ref {
                        // Found valid setup!
                        let mut full_sequence = moves.clone();
                        full_sequence.push(slice_move);
                        full_sequence.extend(get_flipping_macro(size));
                        full_sequence.push(slice_move.inverse());
                        for m in moves.iter().rev() {
                            full_sequence.push(m.inverse());
                        }
                        return Some(full_sequence);
                    }
                }
            }
        }

        if moves.len() >= max_setup_depth {
            continue;
        }

        for &m in &generators {
            let next_src = rotate_coord(curr_src, m, s);
            let next_dest = rotate_coord(curr_dest, m, s);

            if visited.insert((next_src, next_dest)) {
                let mut next_moves = moves.clone();
                next_moves.push(m);
                queue.push_back((next_moves, next_src, next_dest));
            }
        }
    }

    None
}

/// Find outer setup moves to place L2E on FL and FR
fn find_l2e_setup(
    edge1: (Face, Face),
    edge2: (Face, Face),
    size: usize,
) -> Option<Vec<RotationMove>> {
    let s = size as i32;
    let generators = get_outer_generators(size);

    let wings1 = get_edge_wings(edge1.0, edge1.1, size);
    let wings2 = get_edge_wings(edge2.0, edge2.1, size);
    if wings1.is_empty() || wings2.is_empty() {
        return None;
    }
    let start_pos1 = wings1[0];
    let start_pos2 = wings2[0];

    let mut queue = VecDeque::new();
    queue.push_back((Vec::new(), start_pos1, start_pos2));

    let mut visited = HashSet::new();
    visited.insert((start_pos1, start_pos2));

    let max_depth = 4;

    while let Some((moves, p1, p2)) = queue.pop_front() {
        if p1.x == 0 && p1.z == s - 1 && p2.x == s - 1 && p2.z == s - 1 {
            return Some(moves);
        }

        if moves.len() >= max_depth {
            continue;
        }

        for &m in &generators {
            let next_p1 = rotate_coord(p1, m, s);
            let next_p2 = rotate_coord(p2, m, s);

            if visited.insert((next_p1, next_p2)) {
                let mut next_moves = moves.clone();
                next_moves.push(m);
                queue.push_back((next_moves, next_p1, next_p2));
            }
        }
    }

    None
}

/// Standard L2E wing pairing formula moves for layer slice_idx
fn get_l2e_formula_moves(size: usize, slice_idx: i32) -> Vec<RotationMove> {
    let s = size as i32;
    let r_idx = s - 1 - slice_idx;
    let l_idx = slice_idx;

    let r_cw = RotationMove {
        axis: RotationAxis::X,
        index: r_idx,
        direction: Direction::Clockwise,
        add_to_history: true,
    };
    let l_cw = RotationMove {
        axis: RotationAxis::X,
        index: l_idx,
        direction: Direction::Clockwise,
        add_to_history: true,
    };
    let l_ccw = RotationMove {
        axis: RotationAxis::X,
        index: l_idx,
        direction: Direction::CounterClockwise,
        add_to_history: true,
    };

    let u = RotationMove {
        axis: RotationAxis::Y,
        index: s - 1,
        direction: Direction::Clockwise,
        add_to_history: true,
    };
    let f = RotationMove {
        axis: RotationAxis::Z,
        index: s - 1,
        direction: Direction::Clockwise,
        add_to_history: true,
    };

    vec![
        r_cw, u, u, r_cw, u, u, f, f, r_cw, f, f, l_ccw, u, u, l_cw, u, u, r_cw, r_cw,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_edges_already_solved_4x4() {
        let mut state = NxNState::new(4);
        let moves = pair_edges(&mut state);
        assert!(moves.is_some());
    }

    #[test]
    fn test_edges_already_solved_5x5() {
        let mut state = NxNState::new(5);
        let moves = pair_edges(&mut state);
        assert!(moves.is_some());
    }

    #[test]
    fn test_solve_edges_4x4_scrambled() {
        for seed in 1..=3 {
            let mut rng = SimpleRng::new(seed * 300);
            let mut state = NxNState::new(4);

            // Scramble edges with outer moves (so centers remain solved)
            let generators = get_outer_generators(4);
            for _ in 0..5 {
                let idx = rng.next_range(0, generators.len());
                state.apply_move(generators[idx]);
            }

            let solve_result = pair_edges(&mut state);
            assert!(
                solve_result.is_some(),
                "Failed to pair edges for 4x4 with seed {seed}"
            );

            for &(f1, f2) in &COMPOSITE_EDGES {
                assert!(
                    is_edge_paired(&state, f1, f2),
                    "Edge {f1:?} - {f2:?} was not paired with seed {seed}"
                );
            }
        }
    }
}
