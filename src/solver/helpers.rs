use crate::rubik::components::{CubieFace, Direction, Face, RotationAxis, RotationMove};
use crate::rubik::resources::FaceMapping;
use bevy::prelude::*;

pub fn get_cube_state(
    faces: &Query<(&CubieFace, &GlobalTransform)>,
    cube_transform: &GlobalTransform,
    mapping: FaceMapping,
) -> String {
    let mut state = vec![' '; 54];

    let logic_faces = [
        Face::Up,
        Face::Right,
        Face::Front,
        Face::Down,
        Face::Left,
        Face::Back,
    ];

    for (face_idx, &logic_face) in logic_faces.iter().enumerate() {
        let (phys_face, right_vec, down_vec) = mapping.get_face_config(logic_face);
        let normal = phys_face.normal();
        for row in 0..3 {
            for col in 0..3 {
                #[allow(clippy::cast_precision_loss)]
                let i = col as f32 - 1.0;
                #[allow(clippy::cast_precision_loss)]
                let j = row as f32 - 1.0;
                let target_pos = normal * 1.5 + right_vec * i + down_vec * j;

                if let Some(color) =
                    find_facelet_color_at(target_pos, normal, faces, cube_transform)
                {
                    state[face_idx * 9 + row * 3 + col] =
                        mapping.get_char_for_physical_color(color);
                } else {
                    error!(
                        "Missing facelet at {:?} for face {:?}",
                        target_pos, phys_face
                    );
                }
            }
        }
    }

    state.into_iter().collect()
}

fn find_facelet_color_at(
    pos: Vec3,
    normal: Vec3,
    faces: &Query<(&CubieFace, &GlobalTransform)>,
    cube_transform: &GlobalTransform,
) -> Option<Face> {
    let cube_inverse = cube_transform.affine().inverse();
    for (cubie_face, transform) in faces.iter() {
        let face_pos_world = transform.translation();
        let face_pos_local = cube_inverse.transform_point3(face_pos_world);

        let face_normal_world = transform.back();
        let face_normal_local = cube_inverse.transform_vector3(*face_normal_world);

        if face_pos_local.distance(pos) < 0.2 && face_normal_local.dot(normal) > 0.8 {
            return Some(cubie_face.0);
        }
    }
    None
}

#[allow(dead_code)]
pub fn solution_to_moves(solution: &str, size: i32, mapping: FaceMapping) -> Vec<RotationMove> {
    let mut all_moves = Vec::new();
    for part in solution.split_whitespace() {
        let mut chars = part.chars();
        let Some(first_char) = chars.next() else {
            continue;
        };

        // Parse either the new internal slice representation (X{index}, Y{index}, Z{index})
        // or the traditional outer face notations (U, D, L, R, F, B).
        let (axis, index, base_dir, remaining_str) = match first_char {
            'X' | 'Y' | 'Z' => {
                let axis = match first_char {
                    'X' => RotationAxis::X,
                    'Y' => RotationAxis::Y,
                    _ => RotationAxis::Z,
                };

                let mut index_str = String::new();
                let mut modifier_char = None;
                for c in chars {
                    if c.is_ascii_digit() {
                        index_str.push(c);
                    } else {
                        modifier_char = Some(c);
                        break;
                    }
                }
                let Ok(idx) = index_str.parse::<i32>() else {
                    continue;
                };
                (axis, idx, Direction::Clockwise, modifier_char)
            }
            'U' | 'D' | 'L' | 'R' | 'F' | 'B' => {
                let logic_face = match first_char {
                    'U' => Face::Up,
                    'D' => Face::Down,
                    'L' => Face::Left,
                    'R' => Face::Right,
                    'F' => Face::Front,
                    _ => Face::Back,
                };

                let modifier = chars.next();
                let is_inverse = modifier == Some('\'');
                let phys_move = mapping.logic_move_to_physical_move(logic_face, is_inverse, size);

                let is_double = modifier == Some('2');

                all_moves.push(phys_move);
                if is_double {
                    all_moves.push(phys_move);
                }
                continue;
            }
            _ => continue,
        };

        match remaining_str {
            None => all_moves.push(RotationMove {
                axis,
                index,
                direction: base_dir,
                add_to_history: true,
            }),
            Some('\'') => all_moves.push(RotationMove {
                axis,
                index,
                direction: base_dir.inverse(),
                add_to_history: true,
            }),
            Some('2') => {
                let m = RotationMove {
                    axis,
                    index,
                    direction: base_dir,
                    add_to_history: true,
                };
                all_moves.push(m);
                all_moves.push(m);
            }
            _ => {}
        }
    }
    all_moves
}

/// Convert a `RotationMove` into a standard string representation, respecting the Rubik's cube size and `FaceMapping`.
#[allow(dead_code)]
pub fn move_to_string(m: RotationMove, size: i32, mapping: FaceMapping) -> String {
    mapping.physical_move_to_logic_string(m, size)
}

/// Retrieve the state for the Rubik solver, supporting both 3x3x3 (native) and 2x2x2 (mapped to virtual 3x3x3).
pub fn get_cube_state_for_size(
    size: i32,
    faces: &Query<(&CubieFace, &GlobalTransform)>,
    cube_transform: &GlobalTransform,
    mapping: FaceMapping,
) -> Option<String> {
    if size == 3 {
        Some(get_cube_state(faces, cube_transform, mapping))
    } else if size == 2 {
        let mut state = vec![' '; 54];

        let logic_faces = [
            Face::Up,
            Face::Right,
            Face::Front,
            Face::Down,
            Face::Left,
            Face::Back,
        ];

        for (face_idx, &logic_face) in logic_faces.iter().enumerate() {
            let (phys_face, right_vec, down_vec) = mapping.get_face_config(logic_face);
            let normal = phys_face.normal();

            // Set centers and edges to default face color
            let default_char = mapping.get_char_for_physical_color(phys_face);
            for i in [1, 3, 4, 5, 7] {
                state[face_idx * 9 + i] = default_char;
            }

            // Map 2x2x2 corners to 3x3x3 virtual corners
            for row in 0..2 {
                for col in 0..2 {
                    #[allow(clippy::cast_precision_loss)]
                    let i = (col as f32 - 0.5) * 1.53;
                    #[allow(clippy::cast_precision_loss)]
                    let j = (row as f32 - 0.5) * 1.53;
                    let target_pos = normal * 1.5 + right_vec * i + down_vec * j;

                    if let Some(color) =
                        find_facelet_color_at(target_pos, normal, faces, cube_transform)
                    {
                        // Mapping 2x2x2 coordinates to virtual 3x3x3 corners:
                        // (row=0, col=0) -> 3x3x3 corner (row=0, col=0) i.e. index 0
                        // (row=0, col=1) -> 3x3x3 corner (row=0, col=2) i.e. index 2
                        // (row=1, col=0) -> 3x3x3 corner (row=2, col=0) i.e. index 6
                        // (row=1, col=1) -> 3x3x3 corner (row=2, col=2) i.e. index 8
                        let virtual_row = row * 2;
                        let virtual_col = col * 2;
                        let virtual_idx = face_idx * 9 + virtual_row * 3 + virtual_col;
                        state[virtual_idx] = mapping.get_char_for_physical_color(color);
                    } else {
                        error!(
                            "Missing facelet at {:?} for face {:?}",
                            target_pos, phys_face
                        );
                        return None;
                    }
                }
            }
        }

        Some(state.into_iter().collect())
    } else {
        None
    }
}

/// Optimize a sequence of moves by cancelling out and merging adjacent/commutative rotations.
pub fn optimize_moves(moves: &[RotationMove]) -> Vec<RotationMove> {
    let mut optimized: Vec<RotationMove> = Vec::new();
    for &mv in moves {
        let mut merged = false;
        // Search backwards from the end of the stack for a mergeable move
        for i in (0..optimized.len()).rev() {
            let prev = optimized[i];
            if prev.axis != mv.axis {
                // Met a blocker on a different axis, cannot commute past it. Stop searching.
                break;
            }
            if prev.index == mv.index {
                // Found a move on the same axis and slice. Merge them.
                let last_val = match prev.direction {
                    Direction::Clockwise => 1,
                    Direction::CounterClockwise => -1,
                };
                let mv_val = match mv.direction {
                    Direction::Clockwise => 1,
                    Direction::CounterClockwise => -1,
                };
                let total = (last_val + mv_val) % 4;
                let total = if total < 0 { total + 4 } else { total };

                optimized.remove(i); // Remove old move

                match total {
                    1 => {
                        optimized.insert(
                            i,
                            RotationMove {
                                axis: mv.axis,
                                index: mv.index,
                                direction: Direction::Clockwise,
                                add_to_history: true,
                            },
                        );
                    }
                    2 => {
                        let m = RotationMove {
                            axis: mv.axis,
                            index: mv.index,
                            direction: Direction::Clockwise,
                            add_to_history: true,
                        };
                        optimized.insert(i, m);
                        optimized.insert(i + 1, m);
                    }
                    3 => {
                        optimized.insert(
                            i,
                            RotationMove {
                                axis: mv.axis,
                                index: mv.index,
                                direction: Direction::CounterClockwise,
                                add_to_history: true,
                            },
                        );
                    }
                    _ => {
                        // 0, cancelled out completely, already removed so do nothing
                    }
                }
                merged = true;
                break;
            }
        }
        if !merged {
            optimized.push(mv);
        }
    }
    optimized
}

/// Convert any physical `RotationMove` into a logical string representation, respecting `FaceMapping` and cube size.
/// This correctly maps internal slices (as "LX", "LY", "LZ" logic moves) and outer faces (as "U", "D", "R"...) dynamically.
#[allow(clippy::similar_names)]
pub fn physical_move_to_logical_string_any(
    m: RotationMove,
    size: i32,
    mapping: FaceMapping,
) -> String {
    let f_normal = mapping.f_face.normal();
    let d_normal = mapping.d_face.normal();
    let r_normal = f_normal.cross(d_normal);

    // Logic axes vectors in physical space
    let v_x_logic = r_normal;
    let v_y_logic = -d_normal;
    let v_z_logic = f_normal;

    let v_phys = m.axis.vector();

    let dot_x = v_phys.dot(v_x_logic);
    let dot_y = v_phys.dot(v_y_logic);
    let dot_z = v_phys.dot(v_z_logic);

    let (logic_axis, logic_index, logic_direction) = if dot_x.abs() > 0.9 {
        if dot_x > 0.9 {
            (RotationAxis::X, m.index, m.direction)
        } else {
            (RotationAxis::X, size - 1 - m.index, m.direction.inverse())
        }
    } else if dot_y.abs() > 0.9 {
        if dot_y > 0.9 {
            (RotationAxis::Y, m.index, m.direction)
        } else {
            (RotationAxis::Y, size - 1 - m.index, m.direction.inverse())
        }
    } else if dot_z.abs() > 0.9 {
        if dot_z > 0.9 {
            (RotationAxis::Z, m.index, m.direction)
        } else {
            (RotationAxis::Z, size - 1 - m.index, m.direction.inverse())
        }
    } else {
        (m.axis, m.index, m.direction)
    };

    // Format as logical string representation
    if logic_index == size - 1 || logic_index == 0 {
        // Outer face moves
        let base = match logic_axis {
            RotationAxis::X => {
                if logic_index == size - 1 {
                    "R"
                } else {
                    "L"
                }
            }
            RotationAxis::Y => {
                if logic_index == size - 1 {
                    "U"
                } else {
                    "D"
                }
            }
            RotationAxis::Z => {
                if logic_index == size - 1 {
                    "F"
                } else {
                    "B"
                }
            }
        };

        let base_dir = if logic_index == size - 1 {
            Direction::Clockwise
        } else {
            Direction::CounterClockwise
        };

        if logic_direction == base_dir {
            base.to_string()
        } else {
            format!("{base}'")
        }
    } else {
        // Inner slice moves
        let axis_char = match logic_axis {
            RotationAxis::X => "LX",
            RotationAxis::Y => "LY",
            RotationAxis::Z => "LZ",
        };

        if logic_direction == Direction::Clockwise {
            format!("{axis_char}{logic_index}")
        } else {
            format!("{axis_char}{logic_index}'")
        }
    }
}

/// Convert any logical solution string (containing both outer face moves like "U", "D" and inner logic slice moves like "LX1", "LY2")
/// back to physical `RotationMove`s, dynamically respecting the current `FaceMapping` and cube size.
#[allow(clippy::too_many_lines, clippy::similar_names)]
pub fn logical_string_to_physical_moves_any(
    solution: &str,
    size: i32,
    mapping: FaceMapping,
) -> Vec<RotationMove> {
    let mut all_moves = Vec::new();
    let f_normal = mapping.f_face.normal();
    let d_normal = mapping.d_face.normal();
    let r_normal = f_normal.cross(d_normal);

    // Logic axes vectors in physical space
    let v_x_logic = r_normal;
    let v_y_logic = -d_normal;
    let v_z_logic = f_normal;

    for part in solution.split_whitespace() {
        let mut chars = part.chars();
        let Some(first_char) = chars.next() else {
            continue;
        };

        if first_char == 'L' {
            if let Some(second_char) = chars.next() {
                if second_char == 'X' || second_char == 'Y' || second_char == 'Z' {
                    let logic_axis = match second_char {
                        'X' => RotationAxis::X,
                        'Y' => RotationAxis::Y,
                        _ => RotationAxis::Z,
                    };

                    let mut index_str = String::new();
                    let mut modifier_char = None;
                    for c in chars {
                        if c.is_ascii_digit() {
                            index_str.push(c);
                        } else {
                            modifier_char = Some(c);
                            break;
                        }
                    }

                    let Ok(logic_index) = index_str.parse::<i32>() else {
                        continue;
                    };

                    let logic_direction = if modifier_char == Some('\'') {
                        Direction::CounterClockwise
                    } else {
                        Direction::Clockwise
                    };

                    // Map logic move to physical move
                    let v_logic = match logic_axis {
                        RotationAxis::X => v_x_logic,
                        RotationAxis::Y => v_y_logic,
                        RotationAxis::Z => v_z_logic,
                    };

                    let dot_x = v_logic.dot(Vec3::X);
                    let dot_y = v_logic.dot(Vec3::Y);
                    let dot_z = v_logic.dot(Vec3::Z);

                    let (phys_axis, phys_index, phys_direction) = if dot_x.abs() > 0.9 {
                        if dot_x > 0.9 {
                            (RotationAxis::X, logic_index, logic_direction)
                        } else {
                            (
                                RotationAxis::X,
                                size - 1 - logic_index,
                                logic_direction.inverse(),
                            )
                        }
                    } else if dot_y.abs() > 0.9 {
                        if dot_y > 0.9 {
                            (RotationAxis::Y, logic_index, logic_direction)
                        } else {
                            (
                                RotationAxis::Y,
                                size - 1 - logic_index,
                                logic_direction.inverse(),
                            )
                        }
                    } else if dot_z.abs() > 0.9 {
                        if dot_z > 0.9 {
                            (RotationAxis::Z, logic_index, logic_direction)
                        } else {
                            (
                                RotationAxis::Z,
                                size - 1 - logic_index,
                                logic_direction.inverse(),
                            )
                        }
                    } else {
                        continue;
                    };

                    all_moves.push(RotationMove {
                        axis: phys_axis,
                        index: phys_index,
                        direction: phys_direction,
                        add_to_history: true,
                    });
                    continue;
                }
            }
        }

        // Fallback or traditional outer face notations (U, D, L, R, F, B) or old X/Y/Z notations
        let mut chars = part.chars();
        let Some(first_char) = chars.next() else {
            continue;
        };

        match first_char {
            'X' | 'Y' | 'Z' => {
                let axis = match first_char {
                    'X' => RotationAxis::X,
                    'Y' => RotationAxis::Y,
                    _ => RotationAxis::Z,
                };

                let mut index_str = String::new();
                let mut modifier_char = None;
                for c in chars {
                    if c.is_ascii_digit() {
                        index_str.push(c);
                    } else {
                        modifier_char = Some(c);
                        break;
                    }
                }
                let Ok(idx) = index_str.parse::<i32>() else {
                    continue;
                };

                let is_inverse = modifier_char == Some('\'');
                let is_double = modifier_char == Some('2');
                let direction = if is_inverse {
                    Direction::CounterClockwise
                } else {
                    Direction::Clockwise
                };

                let m = RotationMove {
                    axis,
                    index: idx,
                    direction,
                    add_to_history: true,
                };
                all_moves.push(m);
                if is_double {
                    all_moves.push(m);
                }
            }
            'U' | 'D' | 'L' | 'R' | 'F' | 'B' => {
                let logic_face = match first_char {
                    'U' => Face::Up,
                    'D' => Face::Down,
                    'L' => Face::Left,
                    'R' => Face::Right,
                    'F' => Face::Front,
                    _ => Face::Back,
                };

                let modifier = chars.next();
                let is_inverse = modifier == Some('\'');
                let phys_move = mapping.logic_move_to_physical_move(logic_face, is_inverse, size);

                let is_double = modifier == Some('2');

                all_moves.push(phys_move);
                if is_double {
                    all_moves.push(phys_move);
                }
            }
            _ => {}
        }
    }
    all_moves
}
