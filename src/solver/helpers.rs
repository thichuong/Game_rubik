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
pub fn move_to_string(m: RotationMove, size: i32, mapping: FaceMapping) -> String {
    mapping.physical_move_to_logic_string(m, size)
}
