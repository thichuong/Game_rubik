use crate::rubik::components::{CubieFace, Direction, Face, RotationAxis, RotationMove};
use bevy::prelude::*;

pub fn get_cube_state(
    faces: &Query<(&CubieFace, &GlobalTransform)>,
    cube_transform: &GlobalTransform,
) -> String {
    let mut state = vec![' '; 54];

    // Face configurations: (Face normal, Right vector, Down vector)
    let face_configs = [
        (Face::Up, Vec3::X, Vec3::Z),            // U (+Y)
        (Face::Right, Vec3::NEG_Z, Vec3::NEG_Y), // R (+X)
        (Face::Front, Vec3::X, Vec3::NEG_Y),     // F (+Z)
        (Face::Down, Vec3::X, Vec3::NEG_Z),      // D (-Y)
        (Face::Left, Vec3::Z, Vec3::NEG_Y),      // L (-X)
        (Face::Back, Vec3::NEG_X, Vec3::NEG_Y),  // B (-Z)
    ];

    for (face_idx, (face, right_vec, down_vec)) in face_configs.iter().enumerate() {
        let normal = face.normal();
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
                    state[face_idx * 9 + row * 3 + col] = face_to_char(color);
                } else {
                    error!("Missing facelet at {:?} for face {:?}", target_pos, face);
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

const fn face_to_char(face: Face) -> char {
    match face {
        Face::Up => 'U',
        Face::Down => 'D',
        Face::Right => 'R',
        Face::Left => 'L',
        Face::Front => 'F',
        Face::Back => 'B',
    }
}

pub fn solution_to_moves(solution: &str, size: i32) -> Vec<RotationMove> {
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
            'U' => (
                RotationAxis::Y,
                size - 1,
                Direction::Clockwise,
                chars.next(),
            ),
            'D' => (
                RotationAxis::Y,
                0,
                Direction::CounterClockwise,
                chars.next(),
            ),
            'L' => (
                RotationAxis::X,
                0,
                Direction::CounterClockwise,
                chars.next(),
            ),
            'R' => (
                RotationAxis::X,
                size - 1,
                Direction::Clockwise,
                chars.next(),
            ),
            'F' => (
                RotationAxis::Z,
                size - 1,
                Direction::Clockwise,
                chars.next(),
            ),
            'B' => (
                RotationAxis::Z,
                0,
                Direction::CounterClockwise,
                chars.next(),
            ),
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

/// Convert a `RotationMove` into a standard string representation, respecting the Rubik's cube size.
pub fn move_to_string(m: RotationMove, size: i32) -> String {
    let base = match (m.axis, m.index) {
        (RotationAxis::Y, idx) if idx == size - 1 => "U".to_string(),
        (RotationAxis::Y, 0) => "D".to_string(),
        (RotationAxis::X, 0) => "L".to_string(),
        (RotationAxis::X, idx) if idx == size - 1 => "R".to_string(),
        (RotationAxis::Z, idx) if idx == size - 1 => "F".to_string(),
        (RotationAxis::Z, 0) => "B".to_string(),
        (RotationAxis::X, idx) => format!("X{idx}"),
        (RotationAxis::Y, idx) => format!("Y{idx}"),
        (RotationAxis::Z, idx) => format!("Z{idx}"),
    };

    let base_dir = match base.chars().next() {
        Some('D' | 'L' | 'B') => Direction::CounterClockwise,
        _ => Direction::Clockwise,
    };

    if m.direction == base_dir {
        base
    } else {
        format!("{base}'")
    }
}
