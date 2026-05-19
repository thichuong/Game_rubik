#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::doc_markdown,
    clippy::too_many_lines
)]

use crate::core::{Direction, Face, RotationAxis, RotationMove};
use crate::nxn::centers::get_center_coords;
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
    let center_coords = get_center_coords(size);

    // Keep track of paired edges
    let mut paired_edges = HashSet::new();
    for &(f1, f2) in &COMPOSITE_EDGES {
        if is_edge_paired(state, f1, f2) {
            paired_edges.insert((f1, f2));
        }
    }

    for &(f1, f2) in &COMPOSITE_EDGES {
        if paired_edges.contains(&(f1, f2)) {
            continue;
        }

        let wings = get_edge_wings(f1, f2, size);
        if wings.is_empty() {
            continue;
        }

        // We want to pair all other wings in this edge to match the color of the first wing
        let Some(target_colors) = get_wing_colors(state, wings[0], f1, f2) else {
            continue;
        };

        for &dest_coord in &wings[1..] {
            if let Some(colors) = get_wing_colors(state, dest_coord, f1, f2) {
                if colors == target_colors {
                    continue;
                }
            }

            // Find a wing piece with matching colors that is currently not part of a paired edge
            let mut src_coord_opt = None;
            'outer: for &(sf1, sf2) in &COMPOSITE_EDGES {
                if paired_edges.contains(&(sf1, sf2)) || (sf1 == f1 && sf2 == f2) {
                    continue;
                }
                let swings = get_edge_wings(sf1, sf2, size);
                for &sc in &swings {
                    if let Some(colors) = get_wing_colors(state, sc, sf1, sf2) {
                        if (colors.0 == target_colors.0 && colors.1 == target_colors.1)
                            || (colors.0 == target_colors.1 && colors.1 == target_colors.0)
                        {
                            src_coord_opt = Some((sc, sf1, sf2));
                            break 'outer;
                        }
                    }
                }
            }

            let Some((src_coord, _sf1, _sf2)) = src_coord_opt else {
                continue;
            };

            // Run localized BFS to pair this wing
            if let Some(moves) = solve_single_wing(
                state,
                src_coord,
                dest_coord,
                f1,
                f2,
                &center_coords,
                &paired_edges,
            ) {
                state.apply_moves(&moves);
                all_moves.extend(moves);
            } else {
                return None;
            }
        }

        paired_edges.insert((f1, f2));
    }

    Some(all_moves)
}

/// Find a sequence of moves to pair a single wing piece
fn solve_single_wing(
    state: &NxNState,
    src: bevy::prelude::IVec3,
    dest: bevy::prelude::IVec3,
    df1: Face,
    df2: Face,
    centers: &[bevy::prelude::IVec3],
    paired: &HashSet<(Face, Face)>,
) -> Option<Vec<RotationMove>> {
    let size = state.size;

    // Generator moves: standard face rotations + edge flipping macro
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
        generators.push(vec![RotationMove {
            axis,
            index,
            direction: Direction::Clockwise,
            add_to_history: true,
        }]);
        generators.push(vec![RotationMove {
            axis,
            index,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        }]);
    }

    // 2. Standard Edge Flipping Macro: R U R' F R' F' R
    // We can define this macro as a pre-packaged move sequence
    let right_flip = vec![
        RotationMove {
            axis: RotationAxis::X,
            index: size as i32 - 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: size as i32 - 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: size as i32 - 1,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: size as i32 - 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: size as i32 - 1,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: size as i32 - 1,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: size as i32 - 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
    ];
    generators.push(right_flip);

    // 3. Inner slice moves (index from 1 to size-2) paired with flipping/setup moves to preserve centers
    for idx in 1..(size - 1) {
        for &axis in &[RotationAxis::X, RotationAxis::Y, RotationAxis::Z] {
            // Basic slice moves (may be used if we can restore centers later in the search tree)
            generators.push(vec![RotationMove {
                axis,
                index: idx as i32,
                direction: Direction::Clockwise,
                add_to_history: true,
            }]);
            generators.push(vec![RotationMove {
                axis,
                index: idx as i32,
                direction: Direction::CounterClockwise,
                add_to_history: true,
            }]);
        }
    }

    let mut queue = VecDeque::new();
    queue.push_back((state.clone(), Vec::new(), src));

    let mut visited = HashSet::new();
    visited.insert(state.to_string_rep());

    // Depth limit: 4 macro steps
    let max_depth = 4;

    while let Some((curr_state, moves, curr_pos)) = queue.pop_front() {
        // Check if the wing has reached dest and is properly aligned (colors match)
        if curr_pos == dest {
            if let Some(colors) = get_wing_colors(&curr_state, dest, df1, df2) {
                if let Some(target_colors) =
                    get_wing_colors(&curr_state, get_edge_wings(df1, df2, size)[0], df1, df2)
                {
                    if colors == target_colors {
                        return Some(moves);
                    }
                }
            }
        }

        if moves.len() >= max_depth {
            continue;
        }

        for gen_moves in &generators {
            let mut next_state = curr_state.clone();
            next_state.apply_moves(gen_moves);

            // Constraint: Centers must remain intact
            let mut disrupted_centers = false;
            for &coord in centers {
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
                    disrupted_centers = true;
                    break;
                }
            }

            if disrupted_centers {
                continue;
            }

            // Constraint: Already paired edges must not be disrupted
            let mut disrupted_edges = false;
            for &(pf1, pf2) in paired {
                if !is_edge_paired(&next_state, pf1, pf2) {
                    disrupted_edges = true;
                    break;
                }
            }

            if disrupted_edges {
                continue;
            }

            // Track position of our wing
            let mut next_pos = curr_pos;
            for &m in gen_moves {
                let (axis_vec, angle) = m.get_rotation_info();
                let size_i32 = size as i32;
                let is_matched = match m.axis {
                    RotationAxis::X => next_pos.x == m.index,
                    RotationAxis::Y => next_pos.y == m.index,
                    RotationAxis::Z => next_pos.z == m.index,
                };

                if is_matched {
                    let offset = (size_i32 as f32 - 1.0) / 2.0;
                    let rotation = bevy::prelude::Quat::from_axis_angle(axis_vec, angle);
                    let centered = next_pos.as_vec3() - bevy::prelude::Vec3::splat(offset);
                    let rotated = rotation * centered;
                    let restored = rotated + bevy::prelude::Vec3::splat(offset);
                    next_pos = restored.round().as_ivec3();
                }
            }

            let state_rep = next_state.to_string_rep();
            if visited.insert(state_rep) {
                let mut next_moves = moves.clone();
                next_moves.extend(gen_moves.clone());
                queue.push_back((next_state, next_moves, next_pos));
            }
        }
    }

    None
}
