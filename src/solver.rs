use crate::components::{CubieFace, Direction, Face, RotationAxis, RotationMove};
use bevy::prelude::*;
use kewb::DataTable;

#[derive(Resource)]
pub struct SolverResource {
    pub table: DataTable,
}

pub fn get_cube_state(faces: &Query<(&CubieFace, &GlobalTransform)>) -> String {
    let mut state = vec![' '; 54];

    // Face configurations: (Face normal, Right vector, Down vector)
    // Kewb uses U, R, F, D, L, B order
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

                if let Some(color) = find_facelet_color_at(target_pos, normal, faces) {
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
) -> Option<Face> {
    for (cubie_face, transform) in faces.iter() {
        let face_pos = transform.translation();
        let face_normal = transform.back(); // Our spawn_face sets +Z as normal

        if face_pos.distance(pos) < 0.2 && face_normal.dot(normal) > 0.8 {
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

/// Convert the solver's string solution to `RotationMove` sequence
pub fn solution_to_moves(solution: &str) -> Vec<RotationMove> {
    let mut all_moves = Vec::new();
    for part in solution.split_whitespace() {
        let mut chars = part.chars();
        let Some(face_char) = chars.next() else {
            continue;
        };
        let modifier = chars.next();

        let (axis, index, base_dir) = match face_char {
            'U' => (RotationAxis::Y, 1, Direction::Clockwise),
            'D' => (RotationAxis::Y, -1, Direction::CounterClockwise),
            'L' => (RotationAxis::X, -1, Direction::CounterClockwise),
            'R' => (RotationAxis::X, 1, Direction::Clockwise),
            'F' => (RotationAxis::Z, 1, Direction::Clockwise),
            'B' => (RotationAxis::Z, -1, Direction::CounterClockwise),
            _ => continue,
        };

        match modifier {
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
