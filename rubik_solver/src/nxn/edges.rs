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
use crate::nxn::formulas;
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

    for idx in 1..(s - 1) {
        match (f1, f2) {
            (Face::Up, Face::Back) | (Face::Back, Face::Up) => {
                wings.push(bevy::prelude::IVec3::new(idx, s - 1, 0))
            }
            (Face::Up, Face::Right) | (Face::Right, Face::Up) => {
                wings.push(bevy::prelude::IVec3::new(s - 1, s - 1, idx))
            }
            (Face::Up, Face::Front) | (Face::Front, Face::Up) => {
                wings.push(bevy::prelude::IVec3::new(idx, s - 1, s - 1))
            }
            (Face::Up, Face::Left) | (Face::Left, Face::Up) => {
                wings.push(bevy::prelude::IVec3::new(0, s - 1, idx))
            }

            (Face::Down, Face::Back) | (Face::Back, Face::Down) => {
                wings.push(bevy::prelude::IVec3::new(idx, 0, 0))
            }
            (Face::Down, Face::Right) | (Face::Right, Face::Down) => {
                wings.push(bevy::prelude::IVec3::new(s - 1, 0, idx))
            }
            (Face::Down, Face::Front) | (Face::Front, Face::Down) => {
                wings.push(bevy::prelude::IVec3::new(idx, 0, s - 1))
            }
            (Face::Down, Face::Left) | (Face::Left, Face::Down) => {
                wings.push(bevy::prelude::IVec3::new(0, 0, idx))
            }

            (Face::Front, Face::Right) | (Face::Right, Face::Front) => {
                wings.push(bevy::prelude::IVec3::new(s - 1, idx, s - 1))
            }
            (Face::Front, Face::Left) | (Face::Left, Face::Front) => {
                wings.push(bevy::prelude::IVec3::new(0, idx, s - 1))
            }
            (Face::Back, Face::Right) | (Face::Right, Face::Back) => {
                wings.push(bevy::prelude::IVec3::new(s - 1, idx, 0))
            }
            (Face::Back, Face::Left) | (Face::Left, Face::Back) => {
                wings.push(bevy::prelude::IVec3::new(0, idx, 0))
            }
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

/// Check if a composite edge is correctly paired (all wings have correct colors matching their home faces f1 and f2)
pub fn is_edge_paired(state: &NxNState, f1: Face, f2: Face) -> bool {
    let wings = get_edge_wings(f1, f2, state.size);
    if wings.is_empty() {
        return true;
    }

    let Some(first_colors) = get_wing_colors(state, wings[0], f1, f2) else {
        return false;
    };

    for &coord in &wings {
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

    for _attempt in 0..5 {
        let mut paired_edges = HashSet::new();
        for &(f1, f2) in &COMPOSITE_EDGES {
            if is_edge_paired(state, f1, f2) {
                paired_edges.insert((f1, f2));
            }
        }

        if paired_edges.len() == 12 {
            return Some(all_moves);
        }

        let mut loop_count = 0;
        let max_loops = 100;
        let mut shuffle_count = 0;
        let max_shuffles = 10;

        while paired_edges.len() < 12 && loop_count < max_loops {
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

                let target_colors = if size % 2 == 1 {
                    let mid_wing = wings[wings.len() / 2];
                    get_wing_colors(state, mid_wing, f1, f2).unwrap_or((f1, f2))
                } else {
                    get_wing_colors(state, wings[0], f1, f2).unwrap_or((f1, f2))
                };

                let mid_idx = wings.len() / 2;
                for idx in 0..wings.len() {
                    if size % 2 == 1 && idx == mid_idx {
                        continue;
                    }
                    let dest_coord = wings[idx];

                    if let Some(colors) = get_wing_colors(state, dest_coord, f1, f2) {
                        if colors == target_colors {
                            continue;
                        }
                    }

                    let mut src_coord_opt = None;
                    'find_src: for &(f_a, f_b) in &COMPOSITE_EDGES {
                        if f_a == f1 && f_b == f2 {
                            continue;
                        }
                        let candidate_wings = get_edge_wings(f_a, f_b, size);
                        let c_mid_idx = candidate_wings.len() / 2;
                        for c_idx in 0..candidate_wings.len() {
                            if size % 2 == 1 && c_idx == c_mid_idx {
                                continue;
                            }
                            let cand = candidate_wings[c_idx];
                            if let Some(cand_colors) = get_wing_colors(state, cand, f_a, f_b) {
                                if cand_colors == target_colors
                                    || (cand_colors.0 == target_colors.1
                                        && cand_colors.1 == target_colors.0)
                                {
                                    src_coord_opt = Some(cand);
                                    break 'find_src;
                                }
                            }
                        }
                    }

                    let Some(src_coord) = src_coord_opt else {
                        continue;
                    };

                    if let Some(moves) = solve_single_wing(
                        state,
                        src_coord,
                        dest_coord,
                        target_colors,
                        size,
                        (f1, f2),
                    ) {
                        state.apply_moves(&moves);
                        all_moves.extend(moves);
                        progress = true;
                        break 'outer_pair;
                    }
                }
            }

            paired_edges.clear();
            for &(f1, f2) in &COMPOSITE_EDGES {
                if is_edge_paired(state, f1, f2) {
                    paired_edges.insert((f1, f2));
                }
            }

            if !progress {
                if paired_edges.len() < 10 && shuffle_count < max_shuffles {
                    shuffle_count += 1;
                    let generators = get_outer_generators(size);
                    let m = generators[(shuffle_count * 7) % generators.len()];
                    state.apply_move(m);
                    all_moves.push(m);

                    paired_edges.clear();
                    for &(f1, f2) in &COMPOSITE_EDGES {
                        if is_edge_paired(state, f1, f2) {
                            paired_edges.insert((f1, f2));
                        }
                    }
                } else {
                    break;
                }
            }
        }

        paired_edges.clear();
        for &(f1, f2) in &COMPOSITE_EDGES {
            if is_edge_paired(state, f1, f2) {
                paired_edges.insert((f1, f2));
            }
        }

        if paired_edges.len() == 12 {
            return Some(all_moves);
        }

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

                let wings_fl = get_edge_wings(Face::Front, Face::Left, size);
                let wings_fr = get_edge_wings(Face::Front, Face::Right, size);

                if !wings_fl.is_empty() && !wings_fr.is_empty() {
                    let ref_idx = if size % 2 == 1 { wings_fl.len() / 2 } else { 0 };
                    if let Some(ref_color) =
                        get_wing_colors(state, wings_fl[ref_idx], Face::Front, Face::Left)
                    {
                        for idx in 0..wings_fl.len() {
                            if idx == ref_idx {
                                continue;
                            }
                            let w_fl = wings_fl[idx];
                            if let Some(c_fl) =
                                get_wing_colors(state, w_fl, Face::Front, Face::Left)
                            {
                                if c_fl != ref_color {
                                    let slice_idx = w_fl.y;

                                    let mut l2e_moves = Vec::new();
                                    l2e_moves.push(RotationMove {
                                        axis: RotationAxis::Z,
                                        index: (size as i32) - 1,
                                        direction: Direction::CounterClockwise,
                                        add_to_history: true,
                                    });
                                    l2e_moves.extend(get_oll_parity_moves(size, slice_idx));
                                    l2e_moves.push(RotationMove {
                                        axis: RotationAxis::Z,
                                        index: (size as i32) - 1,
                                        direction: Direction::Clockwise,
                                        add_to_history: true,
                                    });

                                    state.apply_moves(&l2e_moves);
                                    all_moves.extend(l2e_moves);
                                }
                            }
                        }
                    }
                }

                let mut undo_setup = setup_moves;
                undo_setup.reverse();
                for m in &mut undo_setup {
                    *m = m.inverse();
                }
                state.apply_moves(&undo_setup);
                all_moves.extend(undo_setup);
            }
        }
    }

    let mut all_ok = true;
    for &(f1, f2) in &COMPOSITE_EDGES {
        if !is_edge_paired(state, f1, f2) {
            all_ok = false;
        }
    }

    if !all_ok {
        return None;
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

/// Count the number of currently fully paired composite edges
fn count_paired_edges(state: &NxNState) -> usize {
    COMPOSITE_EDGES
        .iter()
        .filter(|&&(f1, f2)| is_edge_paired(state, f1, f2))
        .count()
}

/// Find all setup and undo moves to place any free (unpaired) edge at the Up-Right (UR) swap position
fn get_all_free_swap_candidates(state: &NxNState) -> Vec<(Vec<RotationMove>, Vec<RotationMove>)> {
    let size = state.size;
    let s_idx = (size - 1) as i32;
    let mut candidates = Vec::new();

    // 1. Scan Up face first
    let up_edges = [
        ((Face::Up, Face::Right), Vec::new(), Vec::new()),
        (
            (Face::Up, Face::Front),
            vec![RotationMove {
                axis: RotationAxis::Y,
                index: s_idx,
                direction: Direction::CounterClockwise,
                add_to_history: true,
            }],
            vec![RotationMove {
                axis: RotationAxis::Y,
                index: s_idx,
                direction: Direction::Clockwise,
                add_to_history: true,
            }],
        ),
        (
            (Face::Up, Face::Left),
            vec![
                RotationMove {
                    axis: RotationAxis::Y,
                    index: s_idx,
                    direction: Direction::Clockwise,
                    add_to_history: true,
                },
                RotationMove {
                    axis: RotationAxis::Y,
                    index: s_idx,
                    direction: Direction::Clockwise,
                    add_to_history: true,
                },
            ],
            vec![
                RotationMove {
                    axis: RotationAxis::Y,
                    index: s_idx,
                    direction: Direction::Clockwise,
                    add_to_history: true,
                },
                RotationMove {
                    axis: RotationAxis::Y,
                    index: s_idx,
                    direction: Direction::Clockwise,
                    add_to_history: true,
                },
            ],
        ),
        (
            (Face::Up, Face::Back),
            vec![RotationMove {
                axis: RotationAxis::Y,
                index: s_idx,
                direction: Direction::Clockwise,
                add_to_history: true,
            }],
            vec![RotationMove {
                axis: RotationAxis::Y,
                index: s_idx,
                direction: Direction::CounterClockwise,
                add_to_history: true,
            }],
        ),
    ];

    for (edge, setup, undo) in &up_edges {
        if !is_edge_paired(state, edge.0, edge.1) {
            candidates.push((setup.clone(), undo.clone()));
        }
    }

    // 2. Scan Down face
    let down_edges = [
        (
            (Face::Down, Face::Right),
            vec![
                RotationMove {
                    axis: RotationAxis::X,
                    index: s_idx,
                    direction: Direction::Clockwise,
                    add_to_history: true,
                },
                RotationMove {
                    axis: RotationAxis::X,
                    index: s_idx,
                    direction: Direction::Clockwise,
                    add_to_history: true,
                },
            ],
        ),
        (
            (Face::Down, Face::Front),
            vec![
                RotationMove {
                    axis: RotationAxis::Z,
                    index: s_idx,
                    direction: Direction::Clockwise,
                    add_to_history: true,
                },
                RotationMove {
                    axis: RotationAxis::Z,
                    index: s_idx,
                    direction: Direction::Clockwise,
                    add_to_history: true,
                },
            ],
        ),
        (
            (Face::Down, Face::Left),
            vec![
                RotationMove {
                    axis: RotationAxis::X,
                    index: 0,
                    direction: Direction::Clockwise,
                    add_to_history: true,
                },
                RotationMove {
                    axis: RotationAxis::X,
                    index: 0,
                    direction: Direction::Clockwise,
                    add_to_history: true,
                },
            ],
        ),
        (
            (Face::Down, Face::Back),
            vec![
                RotationMove {
                    axis: RotationAxis::Z,
                    index: 0,
                    direction: Direction::Clockwise,
                    add_to_history: true,
                },
                RotationMove {
                    axis: RotationAxis::Z,
                    index: 0,
                    direction: Direction::Clockwise,
                    add_to_history: true,
                },
            ],
        ),
    ];

    for (edge, to_up) in &down_edges {
        if !is_edge_paired(state, edge.0, edge.1) {
            let up_equivalent = match edge {
                (Face::Down, Face::Right) => (Face::Up, Face::Right),
                (Face::Down, Face::Front) => (Face::Up, Face::Front),
                (Face::Down, Face::Left) => (Face::Up, Face::Left),
                (Face::Down, Face::Back) => (Face::Up, Face::Back),
                _ => unreachable!(),
            };

            let up_setup_opt = up_edges.iter().find(|(ue, _, _)| *ue == up_equivalent);
            if let Some((_, up_setup, up_undo)) = up_setup_opt {
                let mut full_setup = to_up.clone();
                full_setup.extend(up_setup.clone());

                let mut full_undo = up_undo.clone();
                full_undo.extend(to_up.clone());

                candidates.push((full_setup, full_undo));
            }
        }
    }

    candidates
}

/// BFS to find setup moves for a single wing pairing using only outer face moves
fn solve_single_wing(
    state: &NxNState,
    src: bevy::prelude::IVec3,
    dest: bevy::prelude::IVec3,
    target_colors: (Face, Face),
    size: usize,
    _edge: (Face, Face),
) -> Option<Vec<RotationMove>> {
    let s = size as i32;
    let generators = get_outer_generators(size);

    let mut queue = VecDeque::new();
    queue.push_back((Vec::new(), src, dest));

    let mut visited = HashSet::new();
    visited.insert((src, dest));

    let max_setup_depth = 8;

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

            let mut temp_state = state.clone();
            temp_state.apply_moves(&moves);

            let slice_move = RotationMove {
                axis: RotationAxis::Y,
                index: slice_idx,
                direction: Direction::CounterClockwise,
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
                let count_slot_paired = |s_state: &NxNState| -> usize {
                    let wings = get_edge_wings(_edge.0, _edge.1, size);
                    if wings.is_empty() {
                        return 0;
                    }
                    let anchor_idx = if size % 2 == 1 { wings.len() / 2 } else { 0 };
                    let Some(anchor_colors) =
                        get_wing_colors(s_state, wings[anchor_idx], _edge.0, _edge.1)
                    else {
                        return 0;
                    };

                    let mut count = 0;
                    for idx in 0..wings.len() {
                        if size % 2 == 1 && idx == anchor_idx {
                            continue;
                        }
                        if let Some(colors) = get_wing_colors(s_state, wings[idx], _edge.0, _edge.1)
                        {
                            if colors == anchor_colors {
                                count += 1;
                            }
                        }
                    }
                    count
                };

                let orig_paired = count_paired_edges(state);
                let orig_slot_total = count_slot_paired(state);

                let candidates = get_all_free_swap_candidates(&temp_state);
                let mut best_seq = None;
                let mut best_slot_total = orig_slot_total;

                for (swap_setup, swap_undo) in &candidates {
                    // Case 1: So Le (Nghịch màu - Chuẩn ghép)
                    if colors_at_slice.0 == target_colors.0 && colors_at_slice.1 == target_colors.1
                    {
                        let mut full_sequence = moves.clone();
                        full_sequence.extend(swap_setup.clone());
                        full_sequence.push(slice_move);
                        full_sequence.extend(formulas::get_edge_flip_algo(size));
                        full_sequence.push(slice_move.inverse());
                        full_sequence.extend(swap_undo.clone());
                        for m in moves.iter().rev() {
                            full_sequence.push(m.inverse());
                        }

                        // Simulate to protect paired edges and active edge winglets
                        let mut sim_state = state.clone();
                        sim_state.apply_moves(&full_sequence);
                        let new_paired = count_paired_edges(&sim_state);
                        let new_slot_total = count_slot_paired(&sim_state);

                        let is_progress = new_paired > orig_paired
                            || (new_paired == orig_paired && new_slot_total > best_slot_total);
                        if is_progress {
                            best_slot_total = new_slot_total;
                            best_seq = Some(full_sequence);
                        }
                    }
                    // Case 2: Song Song (Hợp màu - Cần lật FR trước)
                    else if colors_at_slice.0 == target_colors.1
                        && colors_at_slice.1 == target_colors.0
                    {
                        let mut full_sequence = moves.clone();
                        full_sequence.extend(swap_setup.clone());
                        full_sequence.extend(formulas::get_edge_flip_algo(size)); // Lật FR trước
                        full_sequence.push(slice_move);
                        full_sequence.extend(formulas::get_edge_flip_algo(size)); // EDGE_PAIR_STANDARD
                        full_sequence.push(slice_move.inverse());
                        full_sequence.extend(swap_undo.clone());
                        for m in moves.iter().rev() {
                            full_sequence.push(m.inverse());
                        }

                        // Simulate to protect paired edges and active edge winglets
                        let mut sim_state = state.clone();
                        sim_state.apply_moves(&full_sequence);
                        let new_paired = count_paired_edges(&sim_state);
                        let new_slot_total = count_slot_paired(&sim_state);

                        let is_progress = new_paired > orig_paired
                            || (new_paired == orig_paired && new_slot_total > best_slot_total);
                        if is_progress {
                            best_slot_total = new_slot_total;
                            best_seq = Some(full_sequence);
                        }
                    }
                }

                if let Some(seq) = best_seq {
                    return Some(seq);
                }

                // Fallback to legacy logic without swap if no free edge candidate works (extremely rare but safe)
                // Case 1: So Le
                if colors_at_slice.0 == target_colors.0 && colors_at_slice.1 == target_colors.1 {
                    let mut full_sequence = moves.clone();
                    full_sequence.push(slice_move);
                    full_sequence.extend(formulas::get_edge_flip_algo(size));
                    full_sequence.push(slice_move.inverse());
                    for m in moves.iter().rev() {
                        full_sequence.push(m.inverse());
                    }

                    let mut sim_state = state.clone();
                    sim_state.apply_moves(&full_sequence);
                    let new_paired = count_paired_edges(&sim_state);
                    let new_slot_total = count_slot_paired(&sim_state);

                    let is_progress = new_paired > orig_paired
                        || (new_paired == orig_paired && new_slot_total > orig_slot_total);
                    if is_progress {
                        return Some(full_sequence);
                    }
                }
                // Case 2: Song Song
                else if colors_at_slice.0 == target_colors.1
                    && colors_at_slice.1 == target_colors.0
                {
                    let mut full_sequence = moves.clone();
                    full_sequence.extend(formulas::get_edge_flip_algo(size));
                    full_sequence.push(slice_move);
                    full_sequence.extend(formulas::get_edge_flip_algo(size));
                    full_sequence.push(slice_move.inverse());
                    for m in moves.iter().rev() {
                        full_sequence.push(m.inverse());
                    }

                    let mut sim_state = state.clone();
                    sim_state.apply_moves(&full_sequence);
                    let new_paired = count_paired_edges(&sim_state);
                    let new_slot_total = count_slot_paired(&sim_state);

                    let is_progress = new_paired > orig_paired
                        || (new_paired == orig_paired && new_slot_total > orig_slot_total);
                    if is_progress {
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

    let max_depth = 8;

    while let Some((moves, p1, p2)) = queue.pop_front() {
        let at_fl = |p: bevy::prelude::IVec3| p.x == 0 && p.z == s - 1;
        let at_fr = |p: bevy::prelude::IVec3| p.x == s - 1 && p.z == s - 1;
        if (at_fl(p1) && at_fr(p2)) || (at_fr(p1) && at_fl(p2)) {
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

/// Helper to construct edge-preserving slice OLL Parity moves for NxN Rubik
fn get_oll_parity_moves(size: usize, slice_idx: i32) -> Vec<RotationMove> {
    let s = size as i32;
    let r_idx = s - 1 - slice_idx;
    let l_idx = slice_idx;

    let r_cw = RotationMove {
        axis: RotationAxis::X,
        index: r_idx,
        direction: Direction::Clockwise,
        add_to_history: true,
    };
    let r_ccw = RotationMove {
        axis: RotationAxis::X,
        index: r_idx,
        direction: Direction::CounterClockwise,
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

    let u_cw = RotationMove {
        axis: RotationAxis::Y,
        index: s - 1,
        direction: Direction::Clockwise,
        add_to_history: true,
    };
    let b_cw = RotationMove {
        axis: RotationAxis::Z,
        index: 0,
        direction: Direction::Clockwise,
        add_to_history: true,
    };
    let f_cw = RotationMove {
        axis: RotationAxis::Z,
        index: s - 1,
        direction: Direction::Clockwise,
        add_to_history: true,
    };

    vec![
        // r2
        r_cw, r_cw, // B2
        b_cw, b_cw, // U2
        u_cw, u_cw, // l
        l_cw, // U2
        u_cw, u_cw, // r'
        r_ccw, // U2
        u_cw, u_cw, // r
        r_cw, // U2
        u_cw, u_cw, // F2
        f_cw, f_cw, // r
        r_cw, // F2
        f_cw, f_cw, // l'
        l_ccw, // B2
        b_cw, b_cw, // r2
        r_cw, r_cw,
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
