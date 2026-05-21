use crate::core::{Direction, Face, RotationAxis, RotationMove};
use crate::nxn::state::{FACES_ORDER, NxNState};
use crate::nxn::formulas;
use crate::nxn::orbit::get_orbits;
use std::collections::{HashSet, VecDeque};
use bevy::prelude::{IVec3, Quat, Vec3};

pub fn get_center_coords(size: usize) -> Vec<IVec3> {
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

pub fn solve_centers(state: &mut NxNState) -> Option<Vec<RotationMove>> {
    let mut all_moves = Vec::new();
    let size = state.size;

    // Phase 1 & 2: Solve first 4 faces: Left, Right, Down, Back
    let solve_order = [Face::Left, Face::Right, Face::Down, Face::Back];
    let mut solved_faces = HashSet::new();

    for &target_face in &solve_order {
        let moves = solve_face_with_bars(state, target_face, &solved_faces)?;
        all_moves.extend(moves);
        solved_faces.insert(target_face);
    }

    // Phase 3: L2C for Up and Front
    let l2c_moves = solve_l2c(state, Face::Up, Face::Front)?;
    all_moves.extend(l2c_moves);

    Some(all_moves)
}

fn solve_face_with_bars(state: &mut NxNState, target_face: Face, solved_faces: &HashSet<Face>) -> Option<Vec<RotationMove>> {
    let mut all_moves = Vec::new();
    let size = state.size;

    for col_idx in 1..(size - 1) {
        let working_face = [Face::Up, Face::Front, Face::Down, Face::Back, Face::Left, Face::Right]
            .into_iter()
            .find(|&f| f != target_face && !solved_faces.contains(&f))?;

        // 1. Form a bar of target_face color on working_face (Phase 1: Gom mảnh)
        let moves = form_bar_on_working_face(state, target_face, working_face, col_idx, solved_faces)?;
        state.apply_moves(&moves);
        all_moves.extend(moves);

        // 2. Insert the bar into the target face at col_idx (Phase 2: Xếp thanh)
        let moves = insert_bar_to_target(state, working_face, target_face, col_idx, solved_faces)?;
        state.apply_moves(&moves);
        all_moves.extend(moves);
    }
    Some(all_moves)
}

fn form_bar_on_working_face(state: &mut NxNState, target_color: Face, working_face: Face, target_col_idx: usize, solved_faces: &HashSet<Face>) -> Option<Vec<RotationMove>> {
    let mut all_moves = Vec::new();
    let size = state.size;
    let bar_col = 1; // Arbitrary bar column on working face to build the bar

    for row in 1..(size - 1) {
        if size % 2 == 1 && row == size / 2 && bar_col == size / 2 { continue; }
        let dest = NxNState::get_logical_coord(working_face, row, bar_col, size)?;

        if let Some(f) = state.facelets.iter().find(|f| f.coord == dest) {
            if f.color == target_color { continue; }
        }

        let orbit = get_orbit_of_coord(dest, size);
        let src_pos = state.facelets.iter().find(|f| {
            f.color == target_color &&
            orbit.contains(&f.coord) &&
            !solved_faces.contains(&f.face) &&
            !(f.face == working_face && f.coord == dest) &&
            !(f.face == target_color && is_in_solved_bar(f.coord, size, target_color, target_col_idx))
        }).map(|f| f.coord)?;

        let moves = move_piece_to_pos(state, src_pos, dest, solved_faces, target_color)?;
        state.apply_moves(&moves);
        all_moves.extend(moves);
    }
    Some(all_moves)
}

fn is_in_solved_bar(coord: IVec3, size: usize, face: Face, current_col: usize) -> bool {
    let s = size as i32;
    let col = match face {
        Face::Up => coord.x,
        Face::Down => coord.x,
        Face::Left => coord.z,
        Face::Right => s - 1 - coord.z,
        Face::Front => coord.x,
        Face::Back => s - 1 - coord.x,
    };
    (col as usize) < current_col
}

fn get_orbit_of_coord(coord: IVec3, size: usize) -> Vec<IVec3> {
    let orbits = get_orbits(size);
    for o in orbits {
        if o.positions.contains(&coord) {
            return o.positions;
        }
    }
    Vec::new()
}

fn move_piece_to_pos(state: &NxNState, src: IVec3, dest: IVec3, solved: &HashSet<Face>, target_face: Face) -> Option<Vec<RotationMove>> {
    let size = state.size;
    let mut queue = VecDeque::new();
    queue.push_back((Vec::new(), src));
    let mut visited = HashSet::new();
    visited.insert(src);

    let generators = get_safe_generators(size, solved, target_face);

    while let Some((moves, curr)) = queue.pop_front() {
        if curr == dest { return Some(moves); }
        if moves.len() >= 5 { continue; }

        for &m in &generators {
            let next = rotate_coord(curr, m, size as i32);
            if !visited.contains(&next) {
                visited.insert(next);
                let mut next_moves = moves.clone();
                next_moves.push(m);
                queue.push_back((next_moves, next));
            }
        }
    }
    None
}

fn insert_bar_to_target(state: &NxNState, _working_face: Face, target_face: Face, target_col: usize, solved: &HashSet<Face>) -> Option<Vec<RotationMove>> {
    let size = state.size;
    let mut queue = VecDeque::new();
    queue.push_back(Vec::<RotationMove>::new());

    while let Some(moves) = queue.pop_front() {
        let mut test_state = state.clone();
        test_state.apply_moves(&moves);

        // Is bar inserted?
        let mut inserted = true;
        for row in 1..(size - 1) {
            if size % 2 == 1 && row == size / 2 && target_col == size / 2 { continue; }
            let coord = NxNState::get_logical_coord(target_face, row, target_col, size).unwrap();
            if test_state.facelets.iter().find(|f| f.coord == coord).unwrap().color != target_face {
                inserted = false;
                break;
            }
        }

        if inserted {
            // Are solved faces preserved?
            let mut preserved = true;
            for &sf in solved {
                for row in 1..(size - 1) {
                    for col in 1..(size - 1) {
                        if size % 2 == 1 && row == size / 2 && col == size / 2 { continue; }
                        let coord = NxNState::get_logical_coord(sf, row, col, size).unwrap();
                        if test_state.facelets.iter().find(|f| f.coord == coord).unwrap().color != sf {
                            preserved = false;
                            break;
                        }
                    }
                    if !preserved { break; }
                }
                if !preserved { break; }
            }
            // Also preserve already solved bars on target face
            for prev_col in 1..target_col {
                for row in 1..(size - 1) {
                    if size % 2 == 1 && row == size / 2 && prev_col == size / 2 { continue; }
                    let coord = NxNState::get_logical_coord(target_face, row, prev_col, size).unwrap();
                    if test_state.facelets.iter().find(|f| f.coord == coord).unwrap().color != target_face {
                        preserved = false;
                        break;
                    }
                }
                if !preserved { break; }
            }
            if preserved { return Some(moves); }
        }

        if moves.len() >= 4 { continue; }

        let generators = get_safe_generators(size, solved, target_face);
        for &m in &generators {
            let mut next_moves = moves.clone();
            next_moves.push(m);
            queue.push_back(next_moves);
        }
    }
    None
}

fn solve_l2c(state: &mut NxNState, f1: Face, f2: Face) -> Option<Vec<RotationMove>> {
    let size = state.size;
    let mut all_moves = Vec::new();
    let orbits = get_orbits(size);
    for orbit in orbits {
        let orbit_pos: Vec<IVec3> = orbit.positions.iter()
            .filter(|&&p| {
                let f = get_face_of_coord(p, size as i32).unwrap();
                f == f1 || f == f2
            })
            .cloned().collect();

        let moves = solve_orbit_l2c(state, &orbit_pos, f1, f2)?;
        state.apply_moves(&moves);
        all_moves.extend(moves);
    }
    Some(all_moves)
}

fn solve_orbit_l2c(state: &NxNState, orbit_pos: &[IVec3], f1: Face, f2: Face) -> Option<Vec<RotationMove>> {
    let size = state.size;
    let mut queue = VecDeque::new();
    queue.push_back(Vec::new());

    let mut visited = HashSet::new();
    visited.insert(get_orbit_state_string(state, orbit_pos));

    while let Some(moves) = queue.pop_front() {
        let mut test_state = state.clone();
        test_state.apply_moves(&moves);

        if is_orbit_solved(&test_state, orbit_pos, f1, f2) {
            return Some(moves);
        }

        if moves.len() >= 5 { continue; }

        for comm in get_l2c_generators(size) {
            let mut next_moves = moves.clone();
            next_moves.extend(comm);

            let mut next_state = test_state.clone();
            next_state.apply_moves(&next_moves[moves.len()..]);
            let next_state_str = get_orbit_state_string(&next_state, orbit_pos);

            if visited.insert(next_state_str) {
                queue.push_back(next_moves);
            }
        }
    }
    None
}

fn get_orbit_state_string(state: &NxNState, orbit_pos: &[IVec3]) -> String {
    let mut s = String::new();
    for &pos in orbit_pos {
        if let Some(f) = state.facelets.iter().find(|f| f.coord == pos) {
            s.push_str(&format!("{:?}", f.color));
        }
    }
    s
}

fn is_orbit_solved(state: &NxNState, orbit_pos: &[IVec3], f1: Face, f2: Face) -> bool {
    for &pos in orbit_pos {
        let face = get_face_of_coord(pos, state.size as i32).unwrap();
        if let Some(f) = state.facelets.iter().find(|f| f.coord == pos) {
            if f.color != face { return false; }
        }
    }
    true
}

fn get_l2c_generators(size: usize) -> Vec<Vec<RotationMove>> {
    let mut comms = Vec::new();
    for col in 1..(size - 1) {
        if size % 2 == 1 && col == size / 2 {
            comms.push(formulas::get_center_l2c_mid(size));
        } else if col < size / 2 {
            comms.push(formulas::get_center_l2c_left(size));
        } else {
            comms.push(formulas::get_center_l2c_right(size));
        }
    }
    for &face in &[Face::Up, Face::Front] {
        let (axis, index) = match face {
            Face::Up => (RotationAxis::Y, size as i32 - 1),
            Face::Front => (RotationAxis::Z, size as i32 - 1),
            _ => unreachable!(),
        };
        comms.push(vec![RotationMove { axis, index, direction: Direction::Clockwise, add_to_history: true }]);
        comms.push(vec![RotationMove { axis, index, direction: Direction::CounterClockwise, add_to_history: true }]);
    }
    comms
}

fn get_safe_generators(size: usize, solved: &HashSet<Face>, _target_face: Face) -> Vec<RotationMove> {
    let mut moves = Vec::new();
    for &face in &FACES_ORDER {
        if solved.contains(&face) { continue; }
        let (axis, index) = match face {
            Face::Left => (RotationAxis::X, 0),
            Face::Right => (RotationAxis::X, size as i32 - 1),
            Face::Down => (RotationAxis::Y, 0),
            Face::Up => (RotationAxis::Y, size as i32 - 1),
            Face::Back => (RotationAxis::Z, 0),
            Face::Front => (RotationAxis::Z, size as i32 - 1),
        };
        moves.push(RotationMove { axis, index, direction: Direction::Clockwise, add_to_history: true });
        moves.push(RotationMove { axis, index, direction: Direction::CounterClockwise, add_to_history: true });
    }
    for axis in [RotationAxis::X, RotationAxis::Y, RotationAxis::Z] {
        for index in 1..(size as i32 - 1) {
            moves.push(RotationMove { axis, index, direction: Direction::Clockwise, add_to_history: true });
            moves.push(RotationMove { axis, index, direction: Direction::CounterClockwise, add_to_history: true });
        }
    }
    moves
}

fn get_face_of_coord(coord: IVec3, size: i32) -> Option<Face> {
    if coord.x == 0 { Some(Face::Left) }
    else if coord.x == size - 1 { Some(Face::Right) }
    else if coord.y == 0 { Some(Face::Down) }
    else if coord.y == size - 1 { Some(Face::Up) }
    else if coord.z == 0 { Some(Face::Back) }
    else if coord.z == size - 1 { Some(Face::Front) }
    else { None }
}

fn rotate_coord(coord: IVec3, m: RotationMove, size: i32) -> IVec3 {
    let is_matched = match m.axis {
        RotationAxis::X => coord.x == m.index,
        RotationAxis::Y => coord.y == m.index,
        RotationAxis::Z => coord.z == m.index,
    };
    if is_matched {
        let (axis_vec, angle) = m.get_rotation_info();
        let offset = (size as f32 - 1.0) / 2.0;
        let rotation = Quat::from_axis_angle(axis_vec, angle);
        let centered = coord.as_vec3() - Vec3::splat(offset);
        let rotated = rotation * centered;
        let restored = rotated + Vec3::splat(offset);
        restored.round().as_ivec3()
    } else {
        coord
    }
}
