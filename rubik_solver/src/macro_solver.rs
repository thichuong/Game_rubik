// English: All comments in source code must be in English.
// Vietnamese: Trao đổi trong phần chat hoàn toàn bằng Tiếng Việt.

#![allow(
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::struct_field_names,
    clippy::too_many_lines,
    clippy::cast_possible_truncation
)]

use crate::core::{Direction, Face, RotationAxis, RotationMove};
use bevy::prelude::*;

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, PartialEq)]
pub struct VirtualCubie {
    pub pos: IVec3,
    pub rotation: Quat,
}

impl Eq for VirtualCubie {}
impl Hash for VirtualCubie {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pos.x.hash(state);
        self.pos.y.hash(state);
        self.pos.z.hash(state);
        self.rotation.x.to_bits().hash(state);
        self.rotation.y.to_bits().hash(state);
        self.rotation.z.to_bits().hash(state);
        self.rotation.w.to_bits().hash(state);
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct VirtualCube {
    pub size: i32,
    pub cubies: Vec<VirtualCubie>,
}

fn get_24_rotations() -> Vec<Quat> {
    let mut rotations = Vec::new();
    let angles = [
        0.0,
        std::f32::consts::FRAC_PI_2,
        std::f32::consts::PI,
        3.0 * std::f32::consts::FRAC_PI_2,
    ];
    for &x in &angles {
        for &y in &angles {
            for &z in &angles {
                let q = (Quat::from_rotation_x(x)
                    * Quat::from_rotation_y(y)
                    * Quat::from_rotation_z(z))
                .normalize();
                if !rotations
                    .iter()
                    .any(|existing: &Quat| existing.dot(q).abs() > 0.999)
                {
                    rotations.push(q);
                }
            }
        }
    }
    rotations
}

impl VirtualCube {
    /// Create a solved virtual cube of size N
    pub fn new(size: i32) -> Self {
        let mut cubies = Vec::new();
        for x in 0..size {
            for y in 0..size {
                for z in 0..size {
                    if x > 0 && x < size - 1 && y > 0 && y < size - 1 && z > 0 && z < size - 1 {
                        continue;
                    }
                    cubies.push(VirtualCubie {
                        pos: IVec3::new(x, y, z),
                        rotation: Quat::IDENTITY,
                    });
                }
            }
        }
        Self { size, cubies }
    }

    /// Reconstruct scrambled `VirtualCube` from dynamic flat `state_str`
    pub fn from_state_str(size: i32, state_str: &str) -> Option<Self> {
        let size_usize = size as usize;
        let expected_len = 6 * size_usize * size_usize;
        if state_str.len() != expected_len {
            return None;
        }

        let chars = state_str.as_bytes();

        let get_char = |face_idx: usize, row: usize, col: usize| -> char {
            chars[face_idx * size_usize * size_usize + row * size_usize + col] as char
        };

        let face_for_char = |c: char| -> Option<Face> {
            match c {
                'U' => Some(Face::Up),
                'D' => Some(Face::Down),
                'R' => Some(Face::Right),
                'L' => Some(Face::Left),
                'F' => Some(Face::Front),
                'B' => Some(Face::Back),
                _ => None,
            }
        };

        let mut cubies = Vec::new();
        let rotations_24 = get_24_rotations();

        for x in 0..size {
            for y in 0..size {
                for z in 0..size {
                    if x > 0 && x < size - 1 && y > 0 && y < size - 1 && z > 0 && z < size - 1 {
                        continue;
                    }

                    let mut visible_faces = Vec::new();

                    if x == size - 1 {
                        let col = size - 1 - z;
                        let row = size - 1 - y;
                        let c = get_char(1, row as usize, col as usize);
                        visible_faces.push((Vec3::X, c));
                    }
                    if x == 0 {
                        let col = z;
                        let row = size - 1 - y;
                        let c = get_char(4, row as usize, col as usize);
                        visible_faces.push((Vec3::NEG_X, c));
                    }
                    if y == size - 1 {
                        let col = x;
                        let row = z;
                        let c = get_char(0, row as usize, col as usize);
                        visible_faces.push((Vec3::Y, c));
                    }
                    if y == 0 {
                        let col = x;
                        let row = size - 1 - z;
                        let c = get_char(3, row as usize, col as usize);
                        visible_faces.push((Vec3::NEG_Y, c));
                    }
                    if z == size - 1 {
                        let col = x;
                        let row = size - 1 - y;
                        let c = get_char(2, row as usize, col as usize);
                        visible_faces.push((Vec3::Z, c));
                    }
                    if z == 0 {
                        let col = size - 1 - x;
                        let row = size - 1 - y;
                        let c = get_char(5, row as usize, col as usize);
                        visible_faces.push((Vec3::NEG_Z, c));
                    }

                    let mut found_rot = None;
                    for &q in &rotations_24 {
                        let mut matches = true;
                        for &(normal, c) in &visible_faces {
                            if let Some(expected_logical_face) = face_for_char(c) {
                                let local_dir = q.inverse() * normal;
                                if Face::from_normal(local_dir) != Some(expected_logical_face) {
                                    matches = false;
                                    break;
                                }
                            } else {
                                matches = false;
                                break;
                            }
                        }
                        if matches {
                            found_rot = Some(q);
                            break;
                        }
                    }

                    let rotation = found_rot?;
                    cubies.push(VirtualCubie {
                        pos: IVec3::new(x, y, z),
                        rotation,
                    });
                }
            }
        }

        Some(Self { size, cubies })
    }

    /// Apply a single `RotationMove`
    pub fn apply_move(&mut self, m: RotationMove) {
        let size = self.size;
        let (axis_vec, angle) = m.get_rotation_info();
        let rot_step = Quat::from_axis_angle(axis_vec, angle);
        let offset = (size as f32 - 1.0) / 2.0;

        for cubie in &mut self.cubies {
            if m.is_cubie_at_slice(cubie.pos) {
                let centered = cubie.pos.as_vec3() - Vec3::splat(offset);
                let rotated = rot_step * centered;
                cubie.pos = (rotated + Vec3::splat(offset)).round().as_ivec3();
                cubie.rotation = (rot_step * cubie.rotation).normalize();
            }
        }
    }

    /// Apply a slice of `RotationMoves`
    pub fn apply_moves(&mut self, moves: &[RotationMove]) {
        for &m in moves {
            self.apply_move(m);
        }
    }

    /// Calculate total misplaced stickers
    pub fn count_misplaced_stickers(&self) -> usize {
        let size = self.size;
        let mut count = 0;

        for cubie in &self.cubies {
            if cubie.pos.x == size - 1 {
                let local_dir = cubie.rotation.inverse() * Vec3::X;
                if Face::from_normal(local_dir) != Some(Face::Right) {
                    count += 1;
                }
            }
            if cubie.pos.x == 0 {
                let local_dir = cubie.rotation.inverse() * Vec3::NEG_X;
                if Face::from_normal(local_dir) != Some(Face::Left) {
                    count += 1;
                }
            }
            if cubie.pos.y == size - 1 {
                let local_dir = cubie.rotation.inverse() * Vec3::Y;
                if Face::from_normal(local_dir) != Some(Face::Up) {
                    count += 1;
                }
            }
            if cubie.pos.y == 0 {
                let local_dir = cubie.rotation.inverse() * Vec3::NEG_Y;
                if Face::from_normal(local_dir) != Some(Face::Down) {
                    count += 1;
                }
            }
            if cubie.pos.z == size - 1 {
                let local_dir = cubie.rotation.inverse() * Vec3::Z;
                if Face::from_normal(local_dir) != Some(Face::Front) {
                    count += 1;
                }
            }
            if cubie.pos.z == 0 {
                let local_dir = cubie.rotation.inverse() * Vec3::NEG_Z;
                if Face::from_normal(local_dir) != Some(Face::Back) {
                    count += 1;
                }
            }
        }
        count
    }
}

const fn count_boundary_components(pos: IVec3, size: i32) -> usize {
    let mut count = 0;
    if pos.x == 0 || pos.x == size - 1 {
        count += 1;
    }
    if pos.y == 0 || pos.y == size - 1 {
        count += 1;
    }
    if pos.z == 0 || pos.z == size - 1 {
        count += 1;
    }
    count
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CubeRotation {
    pub x_map: (RotationAxis, bool),
    pub y_map: (RotationAxis, bool),
    pub z_map: (RotationAxis, bool),
}

impl CubeRotation {
    const fn transform_move(self, m: RotationMove, size: i32) -> RotationMove {
        let (new_axis, positive) = match m.axis {
            RotationAxis::X => self.x_map,
            RotationAxis::Y => self.y_map,
            RotationAxis::Z => self.z_map,
        };

        let (new_index, new_direction) = if positive {
            (m.index, m.direction)
        } else {
            (size - 1 - m.index, m.direction.inverse())
        };

        RotationMove {
            axis: new_axis,
            index: new_index,
            direction: new_direction,
            add_to_history: m.add_to_history,
        }
    }
}

const fn get_vector(axis: RotationAxis, positive: bool) -> IVec3 {
    let sign = if positive { 1 } else { -1 };
    match axis {
        RotationAxis::X => IVec3::new(sign, 0, 0),
        RotationAxis::Y => IVec3::new(0, sign, 0),
        RotationAxis::Z => IVec3::new(0, 0, sign),
    }
}

const fn cross(a: IVec3, b: IVec3) -> IVec3 {
    IVec3::new(
        a.y * b.z - a.z * b.y,
        a.z * b.x - a.x * b.z,
        a.x * b.y - a.y * b.x,
    )
}

const fn dot(a: IVec3, b: IVec3) -> i32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

pub fn generate_cube_rotations() -> Vec<CubeRotation> {
    let mut rotations = Vec::with_capacity(24);
    let axes = [RotationAxis::X, RotationAxis::Y, RotationAxis::Z];
    let signs = [true, false];

    for &ax in &axes {
        for &ay in &axes {
            if ax == ay {
                continue;
            }
            for &az in &axes {
                if ax == az || ay == az {
                    continue;
                }
                for &sx in &signs {
                    for &sy in &signs {
                        for &sz in &signs {
                            let ux = get_vector(ax, sx);
                            let uy = get_vector(ay, sy);
                            let uz = get_vector(az, sz);

                            if dot(ux, cross(uy, uz)) == 1 {
                                rotations.push(CubeRotation {
                                    x_map: (ax, sx),
                                    y_map: (ay, sy),
                                    z_map: (az, sz),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    rotations
}

#[derive(Clone)]
pub struct Macro {
    pub name: String,
    pub moves: Vec<RotationMove>,
    pub cost: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymmetricMacro {
    pub name: String,
    pub moves: Vec<RotationMove>,
    pub cost: usize,
}

pub fn generate_symmetric_macros(
    base_macros: &[Macro],
    rotations: &[CubeRotation],
    size: i32,
) -> Vec<SymmetricMacro> {
    let mut sym_macros = Vec::new();
    let mut seen_moves = HashSet::new();

    for base in base_macros {
        for (i, rot) in rotations.iter().enumerate() {
            let mut transformed_moves = Vec::with_capacity(base.moves.len());
            for &m in &base.moves {
                transformed_moves.push(rot.transform_move(m, size));
            }

            if seen_moves.insert(transformed_moves.clone()) {
                sym_macros.push(SymmetricMacro {
                    name: format!("{}_rot{}", base.name, i),
                    moves: transformed_moves,
                    cost: base.cost,
                });
            }
        }
    }
    sym_macros
}

// Base generalized moves generator functions
pub fn get_center1_moves(size: i32, i: i32, j: i32) -> Vec<RotationMove> {
    let r_idx = size - 1 - j;
    let l_idx = i;
    let u_idx = size - 1;
    vec![
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: l_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: l_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
    ]
}

pub fn get_center2_moves(size: i32, i: i32, j: i32) -> Vec<RotationMove> {
    let r_idx = size - 1 - j;
    let l_idx = i;
    let u_idx = size - 1;
    vec![
        RotationMove {
            axis: RotationAxis::X,
            index: l_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: l_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
    ]
}

pub fn get_center3_moves(size: i32, i: i32, j: i32) -> Vec<RotationMove> {
    let f_idx = size - 1 - j;
    let b_idx = i;
    let u_idx = size - 1;
    vec![
        RotationMove {
            axis: RotationAxis::Z,
            index: f_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: b_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: f_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: b_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
    ]
}

pub fn get_center4_moves(size: i32, i: i32, j: i32) -> Vec<RotationMove> {
    let f_idx = size - 1 - j;
    let b_idx = i;
    let u_idx = size - 1;
    vec![
        RotationMove {
            axis: RotationAxis::Z,
            index: b_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: f_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: b_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: f_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
    ]
}

pub fn get_edge_pair_moves(size: i32, i: i32) -> Vec<RotationMove> {
    let u_idx = size - 1;
    let r_idx = size - 1;
    let f_idx = size - 1;
    let slice_idx = i;
    vec![
        RotationMove {
            axis: RotationAxis::Y,
            index: slice_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: f_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: f_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: slice_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
    ]
}

pub fn get_edge_flip_moves(size: i32) -> Vec<RotationMove> {
    let u_idx = size - 1;
    let r_idx = size - 1;
    let f_idx = size - 1;
    vec![
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: f_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: f_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
    ]
}

pub fn get_last_two_edges_1_moves(size: i32, i: i32) -> Vec<RotationMove> {
    let u_idx = size - 1;
    let r_idx = size - 1;
    let f_idx = size - 1;
    let slice_idx = i;
    vec![
        RotationMove {
            axis: RotationAxis::Y,
            index: slice_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: f_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: f_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: slice_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
    ]
}

pub fn get_last_two_edges_2_moves(size: i32, i: i32) -> Vec<RotationMove> {
    let u_idx = size - 1;
    let l_idx = 0;
    let f_idx = size - 1;
    let slice_idx = i;
    vec![
        RotationMove {
            axis: RotationAxis::Y,
            index: slice_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: l_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: l_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: f_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: l_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: f_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: l_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: slice_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
    ]
}

pub fn get_niklas_8_moves(size: i32) -> Vec<RotationMove> {
    let u_idx = size - 1;
    let r_idx = size - 1;
    let l_idx = 0;
    vec![
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: l_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: l_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
    ]
}

pub fn get_t_perm_moves(size: i32) -> Vec<RotationMove> {
    let u_idx = size - 1;
    let r_idx = size - 1;
    let f_idx = size - 1;
    vec![
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: f_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: f_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
    ]
}

pub fn get_pll_parity_moves(size: i32) -> Vec<RotationMove> {
    let r_idx = size - 2;
    let u_idx = size - 1;
    vec![
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
    ]
}

pub fn get_oll_parity_moves(size: i32) -> Vec<RotationMove> {
    let r_idx = size - 2;
    let l_idx = 1;
    let u_idx = size - 1;
    let b_idx = 0;
    let f_idx = size - 1;
    vec![
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: b_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: b_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: l_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: f_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: f_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: f_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: f_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: l_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: b_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::Z,
            index: b_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        RotationMove {
            axis: RotationAxis::X,
            index: r_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
    ]
}

pub fn get_misplaced_centers_signature(cube: &VirtualCube) -> Vec<(IVec3, Face)> {
    let size = cube.size;
    let mut misplaced = Vec::new();
    for cubie in &cube.cubies {
        if count_boundary_components(cubie.pos, size) == 1 {
            let face_at = if cubie.pos.x == size - 1 {
                Face::Right
            } else if cubie.pos.x == 0 {
                Face::Left
            } else if cubie.pos.y == size - 1 {
                Face::Up
            } else if cubie.pos.y == 0 {
                Face::Down
            } else if cubie.pos.z == size - 1 {
                Face::Front
            } else if cubie.pos.z == 0 {
                Face::Back
            } else {
                continue;
            };

            let normal = face_at.normal();
            let local_dir = cubie.rotation.inverse() * normal;
            if let Some(face_color) = Face::from_normal(local_dir) {
                if face_color != face_at {
                    misplaced.push((cubie.pos, face_color));
                }
            }
        }
    }
    misplaced.sort_by(|a, b| {
        a.0.x.cmp(&b.0.x)
            .then(a.0.y.cmp(&b.0.y))
            .then(a.0.z.cmp(&b.0.z))
    });
    misplaced
}

pub fn generate_center_endgame_table(
    macros: &[SymmetricMacro],
    size: i32,
) -> HashMap<Vec<(IVec3, Face)>, SymmetricMacro> {
    let mut table = HashMap::new();
    let solved_cube = VirtualCube::new(size);

    let mut add_to_table = |moves: Vec<RotationMove>, name: String, cost: usize| {
        let mut cube = solved_cube.clone();
        cube.apply_moves(&moves);
        let sig = get_misplaced_centers_signature(&cube);
        if sig.len() >= 2 && sig.len() <= 8 && !table.contains_key(&sig) {
            let inv_moves = moves.iter().rev().map(|m| m.inverse()).collect();
            table.insert(
                sig,
                SymmetricMacro {
                    name: format!("Solve_{}", name),
                    moves: inv_moves,
                    cost,
                },
            );
        }
    };

    // Single commutator
    for mac in macros {
        if mac.cost == 8 {
            add_to_table(mac.moves.clone(), mac.name.clone(), mac.cost);
        }
    }

    // Outer face turns + Single commutator (Setup moves)
    let outer_turns: Vec<_> = macros
        .iter()
        .filter(|m| m.name.starts_with("Outer_Face_Turn"))
        .collect();

    for setup1 in &outer_turns {
        for mac in macros {
            if mac.cost == 8 {
                // Setup 1
                let moves1 = {
                    let mut m = setup1.moves.clone();
                    m.extend(&mac.moves);
                    m.extend(setup1.moves.iter().map(|mv| mv.inverse()));
                    m
                };
                add_to_table(
                    moves1,
                    format!("{}+{}", setup1.name, mac.name),
                    mac.cost + 2,
                );

                // Setup 1 + Setup 2
                for setup2 in &outer_turns {
                    if setup1.moves[0].axis == setup2.moves[0].axis {
                        continue;
                    }
                    let moves2 = {
                        let mut m = setup1.moves.clone();
                        m.extend(&setup2.moves);
                        m.extend(&mac.moves);
                        m.extend(setup2.moves.iter().map(|mv| mv.inverse()));
                        m.extend(setup1.moves.iter().map(|mv| mv.inverse()));
                        m
                    };
                    add_to_table(
                        moves2,
                        format!("{}+{}+{}", setup1.name, setup2.name, mac.name),
                        mac.cost + 4,
                    );
                }
            }
        }
    }
    table
}

pub fn count_misplaced_centers_on_face(cube: &VirtualCube, face: Face) -> usize {
    let size = cube.size;
    let mut count = 0;
    for cubie in &cube.cubies {
        if count_boundary_components(cubie.pos, size) == 1 {
            let is_on_face = match face {
                Face::Right => cubie.pos.x == size - 1,
                Face::Left => cubie.pos.x == 0,
                Face::Up => cubie.pos.y == size - 1,
                Face::Down => cubie.pos.y == 0,
                Face::Front => cubie.pos.z == size - 1,
                Face::Back => cubie.pos.z == 0,
            };
            if is_on_face {
                let normal = face.normal();
                let local_dir = cubie.rotation.inverse() * normal;
                if Face::from_normal(local_dir) != Some(face) {
                    count += 1;
                }
            }
        }
    }
    count
}

pub fn get_face_solving_order(cube: &VirtualCube) -> Vec<Face> {
    let faces = [
        Face::Up,
        Face::Down,
        Face::Front,
        Face::Back,
        Face::Left,
        Face::Right,
    ];
    let mut densities: Vec<(Face, usize)> = faces
        .iter()
        .map(|&f| {
            let misplaced = count_misplaced_centers_on_face(cube, f);
            let total = (cube.size - 2) * (cube.size - 2);
            (f, total as usize - misplaced)
        })
        .collect();

    // Sort by density descending
    densities.sort_by(|a, b| b.1.cmp(&a.1));

    let mut order = Vec::new();
    let mut remaining: HashSet<Face> = faces.iter().copied().collect();

    while !remaining.is_empty() {
        // Pick densest remaining
        if let Some(&(best_face, _)) = densities.iter().find(|(f, _)| remaining.contains(f)) {
            order.push(best_face);
            remaining.remove(&best_face);

            let opposite = match best_face {
                Face::Up => Face::Down,
                Face::Down => Face::Up,
                Face::Front => Face::Back,
                Face::Back => Face::Front,
                Face::Left => Face::Right,
                Face::Right => Face::Left,
            };

            if remaining.contains(&opposite) {
                order.push(opposite);
                remaining.remove(&opposite);
            }
        } else {
            break;
        }
    }
    order
}

#[allow(clippy::option_if_let_else)]
pub fn count_misplaced_centers_staged(cube: &VirtualCube, order: &[Face]) -> usize {
    let mut misplaced_by_face = [0; 6];
    for (i, &face) in order.iter().enumerate() {
        misplaced_by_face[i] = count_misplaced_centers_on_face(cube, face);
    }

    // Phase 1-4: Solve faces individually in order
    for i in 0..4 {
        if misplaced_by_face[i] > 0 {
            let remaining_unsolved = (i..6).filter(|&j| misplaced_by_face[j] > 0).count();
            return remaining_unsolved * 1000 + misplaced_by_face[i];
        }
    }

    // Phase 5: Last Two Centers (L2C) - Solve faces 5 and 6 together
    let l2c_misplaced = misplaced_by_face[4] + misplaced_by_face[5];
    if l2c_misplaced > 0 {
        return l2c_misplaced;
    }

    0
}

#[derive(Clone)]
pub enum SolverPhase {
    Phase1Centers { order: Vec<Face> },
    Phase2Edges,
    Phase3CornersAndParity,
}

fn evaluate_heuristic(cube: &VirtualCube, phase: &SolverPhase) -> usize {
    match phase {
        SolverPhase::Phase1Centers { order } => count_misplaced_centers_staged(cube, order),
        SolverPhase::Phase2Edges => count_unpaired_edges(cube),
        SolverPhase::Phase3CornersAndParity => cube.count_misplaced_stickers(),
    }
}

pub fn count_misplaced_centers(cube: &VirtualCube) -> usize {
    let size = cube.size;
    let mut count = 0;
    for cubie in &cube.cubies {
        if count_boundary_components(cubie.pos, size) == 1 {
            if cubie.pos.x == size - 1 {
                let local_dir = cubie.rotation.inverse() * Vec3::X;
                if Face::from_normal(local_dir) != Some(Face::Right) {
                    count += 1;
                }
            }
            if cubie.pos.x == 0 {
                let local_dir = cubie.rotation.inverse() * Vec3::NEG_X;
                if Face::from_normal(local_dir) != Some(Face::Left) {
                    count += 1;
                }
            }
            if cubie.pos.y == size - 1 {
                let local_dir = cubie.rotation.inverse() * Vec3::Y;
                if Face::from_normal(local_dir) != Some(Face::Up) {
                    count += 1;
                }
            }
            if cubie.pos.y == 0 {
                let local_dir = cubie.rotation.inverse() * Vec3::NEG_Y;
                if Face::from_normal(local_dir) != Some(Face::Down) {
                    count += 1;
                }
            }
            if cubie.pos.z == size - 1 {
                let local_dir = cubie.rotation.inverse() * Vec3::Z;
                if Face::from_normal(local_dir) != Some(Face::Front) {
                    count += 1;
                }
            }
            if cubie.pos.z == 0 {
                let local_dir = cubie.rotation.inverse() * Vec3::NEG_Z;
                if Face::from_normal(local_dir) != Some(Face::Back) {
                    count += 1;
                }
            }
        }
    }
    count
}

pub fn count_unpaired_edges(cube: &VirtualCube) -> usize {
    let size = cube.size;
    let mut unpaired = 0;

    let find_cubie = |x: i32, y: i32, z: i32| -> Option<&VirtualCubie> {
        cube.cubies.iter().find(|c| c.pos == IVec3::new(x, y, z))
    };

    let edge_zones = [
        ((0, 0), 2),
        ((0, size - 1), 2),
        ((size - 1, 0), 2),
        ((size - 1, size - 1), 2),
        ((0, 0), 1),
        ((0, size - 1), 1),
        ((size - 1, 0), 1),
        ((size - 1, size - 1), 1),
        ((0, 0), 0),
        ((0, size - 1), 0),
        ((size - 1, 0), 0),
        ((size - 1, size - 1), 0),
    ];

    for &((a, b), axis) in &edge_zones {
        for i in 1..(size - 2) {
            let (c1_pos, c2_pos) = match axis {
                0 => (IVec3::new(i, a, b), IVec3::new(i + 1, a, b)),
                1 => (IVec3::new(a, i, b), IVec3::new(a, i + 1, b)),
                _ => (IVec3::new(a, b, i), IVec3::new(a, b, i + 1)),
            };

            if let (Some(c1), Some(c2)) = (
                find_cubie(c1_pos.x, c1_pos.y, c1_pos.z),
                find_cubie(c2_pos.x, c2_pos.y, c2_pos.z),
            ) {
                if c1.rotation.dot(c2.rotation).abs() < 0.99 {
                    unpaired += 2;
                }
            }
        }
    }

    unpaired
}

#[derive(Clone)]
struct SearchNode {
    cube: VirtualCube,
    macro_indices: Vec<usize>,
    total_cost: usize,
}

#[allow(clippy::implicit_hasher)]
pub fn solve_phase_beam_search(
    cube: &VirtualCube,
    phase: &SolverPhase,
    macros: &[SymmetricMacro],
    beam_width: usize,
    max_depth: usize,
    global_visited: &HashSet<VirtualCube>,
) -> Option<Vec<SymmetricMacro>> {
    let initial_heuristic = evaluate_heuristic(cube, phase);
    if initial_heuristic == 0 {
        return Some(Vec::new());
    }

    let mut current_beam = VecDeque::new();
    current_beam.push_back(SearchNode {
        cube: cube.clone(),
        macro_indices: Vec::new(),
        total_cost: 0,
    });

    let mut best_improvement_node: Option<SearchNode> = None;
    let mut best_heuristic = initial_heuristic;
    let mut visited_states = HashSet::new();
    visited_states.insert(cube.clone());

    for _depth in 1..=max_depth {
        let mut next_candidates = Vec::new();

        while let Some(node) = current_beam.pop_front() {
            for (mac_idx, mac) in macros.iter().enumerate() {
                let mut next_cube = node.cube.clone();
                next_cube.apply_moves(&mac.moves);

                if global_visited.contains(&next_cube) || visited_states.contains(&next_cube) {
                    continue;
                }
                visited_states.insert(next_cube.clone());

                let h = evaluate_heuristic(&next_cube, phase);
                let next_cost = node.total_cost + mac.cost;

                let mut next_indices = node.macro_indices.clone();
                next_indices.push(mac_idx);

                let candidate = SearchNode {
                    cube: next_cube,
                    macro_indices: next_indices,
                    total_cost: next_cost,
                };

                if h < best_heuristic {
                    best_heuristic = h;
                    best_improvement_node = Some(candidate.clone());
                } else if h == best_heuristic {
                    if let Some(ref best) = best_improvement_node {
                        if next_cost < best.total_cost {
                            best_improvement_node = Some(candidate.clone());
                        }
                    } else {
                        best_improvement_node = Some(candidate.clone());
                    }
                }

                next_candidates.push((h, next_cost, candidate));
            }
        }

        if next_candidates.is_empty() {
            break;
        }

        next_candidates.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

        current_beam.clear();
        for (_, _, node) in next_candidates.iter().take(beam_width) {
            current_beam.push_back(node.clone());
        }

        if best_heuristic == 0 {
            break;
        }
    }

    if let Some(best_node) = best_improvement_node {
        if best_heuristic >= initial_heuristic {
            return Some(Vec::new());
        }
        let applied_macros = best_node
            .macro_indices
            .iter()
            .map(|&idx| macros[idx].clone())
            .collect();
        Some(applied_macros)
    } else {
        None
    }
}

pub fn solve_cube_macro(cube: &mut VirtualCube) -> Option<Vec<RotationMove>> {
    let size = cube.size;
    let rotations = generate_cube_rotations();

    let mut center_bases = Vec::new();
    // Clockwise Outer Face Turn
    center_bases.push(Macro {
        name: "Outer_Face_Turn_CW".to_string(),
        moves: vec![RotationMove {
            axis: RotationAxis::X,
            index: size - 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        }],
        cost: 1,
    });
    // CounterClockwise Outer Face Turn
    center_bases.push(Macro {
        name: "Outer_Face_Turn_CCW".to_string(),
        moves: vec![RotationMove {
            axis: RotationAxis::X,
            index: size - 1,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        }],
        cost: 1,
    });
    for i in 1..(size - 1) {
        // Clockwise Inner Slice Turn
        center_bases.push(Macro {
            name: format!("Inner_Slice_Turn_CW_s{i}"),
            moves: vec![RotationMove {
                axis: RotationAxis::X,
                index: i,
                direction: Direction::Clockwise,
                add_to_history: true,
            }],
            cost: 1,
        });
        // CounterClockwise Inner Slice Turn
        center_bases.push(Macro {
            name: format!("Inner_Slice_Turn_CCW_s{i}"),
            moves: vec![RotationMove {
                axis: RotationAxis::X,
                index: i,
                direction: Direction::CounterClockwise,
                add_to_history: true,
            }],
            cost: 1,
        });
        for j in 1..(size - 1) {
            center_bases.push(Macro {
                name: format!("Center_F_U_Right_s{i}_s{j}"),
                moves: get_center1_moves(size, i, j),
                cost: 8,
            });
            center_bases.push(Macro {
                name: format!("Center_F_U_Left_s{i}_s{j}"),
                moves: get_center2_moves(size, i, j),
                cost: 8,
            });
            center_bases.push(Macro {
                name: format!("Center_R_U_Back_s{i}_s{j}"),
                moves: get_center3_moves(size, i, j),
                cost: 8,
            });
            center_bases.push(Macro {
                name: format!("Center_R_U_Front_s{i}_s{j}"),
                moves: get_center4_moves(size, i, j),
                cost: 8,
            });
        }
    }
    let center_macros = generate_symmetric_macros(&center_bases, &rotations, size);

    let mut edge_bases = Vec::new();
    edge_bases.push(Macro {
        name: "Outer_Face_Turn".to_string(),
        moves: vec![RotationMove {
            axis: RotationAxis::X,
            index: size - 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        }],
        cost: 1,
    });
    edge_bases.push(Macro {
        name: "Edge_Flip".to_string(),
        moves: get_edge_flip_moves(size),
        cost: 7,
    });
    for i in 1..(size - 1) {
        edge_bases.push(Macro {
            name: format!("Edge_Pair_R_F_s{i}"),
            moves: get_edge_pair_moves(size, i),
            cost: 9,
        });
        edge_bases.push(Macro {
            name: format!("Last_Two_Edges_1_s{i}"),
            moves: get_last_two_edges_1_moves(size, i),
            cost: 9,
        });
        edge_bases.push(Macro {
            name: format!("Last_Two_Edges_2_s{i}"),
            moves: get_last_two_edges_2_moves(size, i),
            cost: 9,
        });
    }
    let edge_macros = generate_symmetric_macros(&edge_bases, &rotations, size);

    let stage3_bases = vec![
        Macro {
            name: "Outer_Face_Turn".to_string(),
            moves: vec![RotationMove {
                axis: RotationAxis::X,
                index: size - 1,
                direction: Direction::Clockwise,
                add_to_history: true,
            }],
            cost: 1,
        },
        Macro {
            name: "Corner_Cycle_Niklas".to_string(),
            moves: get_niklas_8_moves(size),
            cost: 8,
        },
        Macro {
            name: "Corner_Swap_T_Perm".to_string(),
            moves: get_t_perm_moves(size),
            cost: 15,
        },
        Macro {
            name: "PLL_Parity".to_string(),
            moves: get_pll_parity_moves(size),
            cost: 12,
        },
        Macro {
            name: "OLL_Parity".to_string(),
            moves: get_oll_parity_moves(size),
            cost: 25,
        },
        Macro {
            name: "Edge_Flip_Stage3".to_string(),
            moves: get_edge_flip_moves(size),
            cost: 7,
        },
    ];
    let stage3_macros = generate_symmetric_macros(&stage3_bases, &rotations, size);

    let mut solved_solution = Vec::new();

    let total_centers = 6 * (size - 2) * (size - 2);
    let max_center_steps = (total_centers * 2) as usize;
    let total_edges = 12 * (size - 2);
    let max_edge_steps = (total_edges * 2) as usize;
    let max_stage3_steps = 50;

    // Phase 1: Solving Centers (Hybrid: Greedy Shallow Search + Endgame Lookup)
    let mut step = 1;
    let mut global_visited_centers = HashSet::new();
    global_visited_centers.insert(cube.clone());

    let center_endgame_table = generate_center_endgame_table(&center_macros, size);
    let solving_order = get_face_solving_order(cube);
    let center_phase = SolverPhase::Phase1Centers {
        order: solving_order,
    };

    loop {
        let misplaced = count_misplaced_centers(cube);
        if misplaced == 0 {
            break;
        }

        if misplaced <= 8 {
            // Stage 2: Endgame Lookup
            let sig = get_misplaced_centers_signature(cube);
            if let Some(mac) = center_endgame_table.get(&sig) {
                cube.apply_moves(&mac.moves);
                for &mv in &mac.moves {
                    solved_solution.push(mv);
                }
                continue;
            }
        }

        // Stage 1: Greedy Shallow Search
        let mut best_macros = solve_phase_beam_search(
            cube,
            &center_phase,
            &center_macros,
            15, // beam_width = 15
            2,  // max_depth = 2
            &global_visited_centers,
        );

        if let Some(ref bm) = best_macros {
            if bm.is_empty() {
                // Adaptive Fallback: search deeper if stuck
                best_macros = solve_phase_beam_search(
                    cube,
                    &center_phase,
                    &center_macros,
                    300,
                    8,
                    &global_visited_centers,
                );
            }
        }

        if let Some(bm) = best_macros {
            if bm.is_empty() {
                break;
            }
            for m in &bm {
                cube.apply_moves(&m.moves);
                global_visited_centers.insert(cube.clone());
                for &mv in &m.moves {
                    solved_solution.push(mv);
                }
            }
        } else {
            return None;
        }
        step += 1;
        if step > max_center_steps {
            break;
        }
    }

    // Phase 2: Solving Edges
    step = 1;
    let mut global_visited_edges = HashSet::new();
    global_visited_edges.insert(cube.clone());

    loop {
        let unpaired = count_unpaired_edges(cube);
        if unpaired == 0 {
            break;
        }

        let mut best_macros = solve_phase_beam_search(
            cube,
            &SolverPhase::Phase2Edges,
            &edge_macros,
            50,
            5,
            &global_visited_edges,
        );

        if let Some(ref bm) = best_macros {
            if bm.is_empty() {
                // Adaptive Fallback: search deeper if stuck
                best_macros = solve_phase_beam_search(
                    cube,
                    &SolverPhase::Phase2Edges,
                    &edge_macros,
                    300,
                    8,
                    &global_visited_edges,
                );
            }
        }

        if let Some(bm) = best_macros {
            if bm.is_empty() {
                break;
            }
            for m in &bm {
                cube.apply_moves(&m.moves);
                global_visited_edges.insert(cube.clone());
                for &mv in &m.moves {
                    solved_solution.push(mv);
                }
            }
        } else {
            return None;
        }
        step += 1;
        if step > max_edge_steps {
            break;
        }
    }

    // Phase 3: Solving Corners, Edges and Parities
    step = 1;
    let mut global_visited_stage3 = HashSet::new();
    global_visited_stage3.insert(cube.clone());

    loop {
        let misplaced = cube.count_misplaced_stickers();
        if misplaced == 0 {
            break;
        }

        let mut best_macros = solve_phase_beam_search(
            cube,
            &SolverPhase::Phase3CornersAndParity,
            &stage3_macros,
            50,
            6,
            &global_visited_stage3,
        );

        if let Some(ref bm) = best_macros {
            if bm.is_empty() {
                // Adaptive Fallback: search deeper if stuck
                best_macros = solve_phase_beam_search(
                    cube,
                    &SolverPhase::Phase3CornersAndParity,
                    &stage3_macros,
                    300,
                    8,
                    &global_visited_stage3,
                );
            }
        }

        if let Some(bm) = best_macros {
            if bm.is_empty() {
                break;
            }
            for m in &bm {
                cube.apply_moves(&m.moves);
                global_visited_stage3.insert(cube.clone());
                for &mv in &m.moves {
                    solved_solution.push(mv);
                }
            }
        } else {
            return None;
        }
        step += 1;
        if step > max_stage3_steps {
            break;
        }
    }

    if cube.count_misplaced_stickers() == 0 {
        Some(solved_solution)
    } else {
        None
    }
}
