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
use serde::{Deserialize, Serialize};

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
    /// Rotate the entire cube (reorienting it)
    pub fn rotate(&mut self, rot: CubeRotation) {
        let size = self.size;
        let offset = (size as f32 - 1.0) / 2.0;

        let (ax, sx) = rot.x_map;
        let (ay, sy) = rot.y_map;
        let (az, sz) = rot.z_map;

        let ux = get_vector(ax, sx).as_vec3();
        let uy = get_vector(ay, sy).as_vec3();
        let uz = get_vector(az, sz).as_vec3();

        let mat = Mat3::from_cols(ux, uy, uz);
        let q_rot = Quat::from_mat3(&mat);

        for cubie in &mut self.cubies {
            let centered = cubie.pos.as_vec3() - Vec3::splat(offset);
            let rotated = q_rot * centered;
            cubie.pos = (rotated + Vec3::splat(offset)).round().as_ivec3();
            cubie.rotation = (q_rot * cubie.rotation).normalize();
        }
    }

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
    pub fn inverse(self) -> Self {
        // A rotation is a 3x3 orthogonal matrix. Transpose is inverse.
        let ux = get_vector(self.x_map.0, self.x_map.1);
        let uy = get_vector(self.y_map.0, self.y_map.1);
        let uz = get_vector(self.z_map.0, self.z_map.1);

        let mut x_map = (RotationAxis::X, true);
        let mut y_map = (RotationAxis::Y, true);
        let mut z_map = (RotationAxis::Z, true);

        let axes = [(RotationAxis::X, ux), (RotationAxis::Y, uy), (RotationAxis::Z, uz)];

        for (target_axis, vec) in axes {
            if vec.x != 0 {
                x_map = (target_axis, vec.x > 0);
            } else if vec.y != 0 {
                y_map = (target_axis, vec.y > 0);
            } else if vec.z != 0 {
                z_map = (target_axis, vec.z > 0);
            }
        }

        Self { x_map, y_map, z_map }
    }

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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

/// A "pure" commutator for L2C that targets Up and Front faces.
/// Formula: [r, U2] = r U2 r' U2
pub fn get_pure_commutator_u_f(size: i32, r_idx: i32) -> Vec<RotationMove> {
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

#[derive(Serialize, Deserialize)]
pub struct L2CTable {
    pub table: HashMap<u16, Vec<RotationMove>>,
}

impl L2CTable {
    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string(self).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, format!("Serde error: {e}"))
        })?;
        std::fs::write(path, json)
    }

    pub fn load(path: &str) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let table = serde_json::from_str(&json).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, format!("Serde error: {e}"))
        })?;
        Ok(table)
    }

    pub fn generate_for_6x6() -> Self {
        let size = 6;
        Self::generate(size, 4)
    }

    pub fn generate(size: i32, max_depth: usize) -> Self {
        let mut table = HashMap::new();
        let mut queue = VecDeque::new();

        let initial_cube = VirtualCube::new(size);
        let initial_mask = get_face_bitmask(&initial_cube, Face::Up);
        table.insert(initial_mask, Vec::new());
        queue.push_back((initial_cube, Vec::new(), 0));

        let mut base_moves = Vec::new();
        for i in 1..(size - 1) {
            let fw = get_pure_commutator_u_f(size, i);
            let mut bw = fw.clone();
            bw.reverse();
            bw = bw.into_iter().map(|m| m.inverse()).collect();
            base_moves.push(fw);
            base_moves.push(bw);
        }

        base_moves.push(vec![RotationMove {
            axis: RotationAxis::Y,
            index: size - 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        }]);
        base_moves.push(vec![RotationMove {
            axis: RotationAxis::Y,
            index: size - 1,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        }]);

        while let Some((cube, moves, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }
            for base_m in &base_moves {
                let mut next_cube = cube.clone();
                next_cube.apply_moves(base_m);
                let next_mask = get_face_bitmask(&next_cube, Face::Up);

                if !table.contains_key(&next_mask) {
                    let mut next_moves = Vec::new();
                    // Scrambled to solved: inverse of the moves that took us from solved to here
                    let inv_moves: Vec<RotationMove> =
                        base_m.iter().rev().map(|m| m.inverse()).collect();
                    next_moves.extend(inv_moves);
                    next_moves.extend(moves.clone());

                    table.insert(next_mask, next_moves.clone());
                    queue.push_back((next_cube, next_moves, depth + 1));
                }
            }
        }

        Self { table }
    }
}

pub fn get_face_bitmask(cube: &VirtualCube, face: Face) -> u16 {
    let size = cube.size;
    let mut bitmask = 0u16;
    let mut bit_idx = 0;

    let normal = face.normal();

    for row in 1..(size - 1) {
        for col in 1..(size - 1) {
            let (x, y, z) = match face {
                Face::Up => (col, size - 1, row),
                Face::Down => (col, 0, size - 1 - row),
                Face::Left => (0, size - 1 - row, col),
                Face::Right => (size - 1, size - 1 - row, size - 1 - col),
                Face::Front => (col, size - 1 - row, size - 1),
                Face::Back => (size - 1 - col, size - 1 - row, 0),
            };

            if let Some(cubie) = cube.cubies.iter().find(|c| c.pos == IVec3::new(x, y, z)) {
                let local_dir = cubie.rotation.inverse() * normal;
                if Face::from_normal(local_dir) != Some(face) {
                    bitmask |= 1 << bit_idx;
                }
            }
            bit_idx += 1;
        }
    }
    bitmask
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
                let normal = match face {
                    Face::Right => Vec3::X,
                    Face::Left => Vec3::NEG_X,
                    Face::Up => Vec3::Y,
                    Face::Down => Vec3::NEG_Y,
                    Face::Front => Vec3::Z,
                    Face::Back => Vec3::NEG_Z,
                };
                let local_dir = cubie.rotation.inverse() * normal;
                if Face::from_normal(local_dir) != Some(face) {
                    count += 1;
                }
            }
        }
    }
    count
}

#[allow(clippy::option_if_let_else)]
pub fn count_misplaced_centers_staged(cube: &VirtualCube) -> usize {
    let faces = [
        Face::Up,
        Face::Down,
        Face::Left,
        Face::Right,
        Face::Front,
        Face::Back,
    ];
    let mut misplaced = Vec::new();
    for &face in &faces {
        misplaced.push((face, count_misplaced_centers_on_face(cube, face)));
    }

    let unsolved_count = misplaced.iter().filter(|&&(_, m)| m > 0).count();
    if unsolved_count == 0 {
        return 0;
    }

    // If we have more than 2 faces unsolved, we pick the one with the FEWEST misplaced pieces (highest density)
    // to solve first. But we must also consider the "opposite" face strategy.

    // For simplicity, let's find the face with the minimum non-zero misplaced count.
    let mut best_face_idx = None;
    let mut min_misplaced = 999;

    for (i, &(_, m)) in misplaced.iter().enumerate() {
        if m > 0 && m < min_misplaced {
            min_misplaced = m;
            best_face_idx = Some(i);
        }
    }

    if let Some(_) = best_face_idx {
        // We want to prioritize solving faces one by one.
        // Penalty based on number of unsolved faces + misplaced on the "current" target face.
        unsolved_count * 1000 + min_misplaced
    } else {
        0
    }
}

#[derive(Clone, Copy)]
pub enum SolverPhase {
    Phase1Centers,
    Phase2Edges,
    Phase3CornersAndParity,
}

fn evaluate_heuristic(cube: &VirtualCube, phase: SolverPhase) -> usize {
    match phase {
        SolverPhase::Phase1Centers => count_misplaced_centers_staged(cube),
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
    phase: SolverPhase,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_cube_new() {
        let cube = VirtualCube::new(3);
        assert_eq!(cube.size, 3);
        assert_eq!(cube.cubies.len(), 26);
    }

    #[test]
    fn test_get_face_bitmask_solved() {
        let cube = VirtualCube::new(6);
        let mask = get_face_bitmask(&cube, Face::Up);
        assert_eq!(mask, 0);
    }
}

pub fn solve_cube_macro(cube: &mut VirtualCube) -> Option<Vec<RotationMove>> {
    solve_cube_macro_hybrid(cube, None)
}

fn solve_cube_macro_old(cube: &mut VirtualCube) -> Option<Vec<RotationMove>> {
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

    // Phase 1: Solving Centers
    let mut step = 1;
    let mut global_visited_centers = HashSet::new();
    global_visited_centers.insert(cube.clone());

    loop {
        let misplaced = count_misplaced_centers(cube);
        if misplaced == 0 {
            break;
        }

        let mut best_macros = solve_phase_beam_search(
            cube,
            SolverPhase::Phase1Centers,
            &center_macros,
            50,
            5,
            &global_visited_centers,
        );

        if let Some(ref bm) = best_macros {
            if bm.is_empty() {
                // Adaptive Fallback: search deeper if stuck
                best_macros = solve_phase_beam_search(
                    cube,
                    SolverPhase::Phase1Centers,
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
            SolverPhase::Phase2Edges,
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
                    SolverPhase::Phase2Edges,
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
            SolverPhase::Phase3CornersAndParity,
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
                    SolverPhase::Phase3CornersAndParity,
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

pub fn solve_l2c_lookup(
    cube: &VirtualCube,
    table: &L2CTable,
) -> Option<Vec<RotationMove>> {
    // 1. Identify which two faces are unsolved.
    let faces = [
        Face::Up,
        Face::Down,
        Face::Left,
        Face::Right,
        Face::Front,
        Face::Back,
    ];
    let mut unsolved_faces = Vec::new();
    for &f in &faces {
        if count_misplaced_centers_on_face(cube, f) > 0 {
            unsolved_faces.push(f);
        }
    }

    if unsolved_faces.is_empty() {
        return Some(Vec::new());
    }
    if unsolved_faces.len() != 2 {
        // L2C logic only applies when exactly 2 faces are unsolved
        return None;
    }

    // 2. Try all 24 rotations to see if we find a state in the table.
    let rotations = generate_cube_rotations();
    for rot in rotations {
        let mut rotated_cube = cube.clone();
        rotated_cube.rotate(rot);

        let mask = get_face_bitmask(&rotated_cube, Face::Up);
        if let Some(moves) = table.table.get(&mask) {
            // We found the moves! But they are in the rotated orientation.
            // We need to transform them back.
            // Actually, we can just apply them to the original cube by rotating them.
            // If `rot` transforms original to rotated, then `inv_rot` transforms rotated back to original.
            // But it's easier to just rotate the moves from our table's perspective.

            // Wait, if we apply moves `M` to `rotated_cube`, it solves it.
            // `rotated_cube = rot(cube)`
            // `solved = M(rotated_cube) = M(rot(cube))`
            // We want `solved = rot(M'(cube))` where `M'` are the moves to apply to the original cube.
            // `M'(cube) = rot_inv(M(rot(cube)))`

            // To get M', we can transform each move in M using the inverse of `rot`.
            let ir = rot.inverse();
            let transformed_moves = moves.iter().map(|m| ir.transform_move(*m, cube.size)).collect();
            return Some(transformed_moves);
        }
    }

    None
}

pub fn solve_cube_macro_hybrid(
    cube: &mut VirtualCube,
    l2c_table: Option<&L2CTable>,
) -> Option<Vec<RotationMove>> {
    let size = cube.size;
    let rotations = generate_cube_rotations();

    let mut center_bases = Vec::new();
    // Group 1: Destructive Moves (Outer & Inner Slice turns)
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
    }
    let destructive_center_macros = generate_symmetric_macros(&center_bases, &rotations, size);

    let mut center_pure_bases = Vec::new();
    for i in 1..(size - 1) {
        for j in 1..(size - 1) {
            center_pure_bases.push(Macro {
                name: format!("Center_F_U_Right_s{i}_s{j}"),
                moves: get_center1_moves(size, i, j),
                cost: 8,
            });
            center_pure_bases.push(Macro {
                name: format!("Center_F_U_Left_s{i}_s{j}"),
                moves: get_center2_moves(size, i, j),
                cost: 8,
            });
            center_pure_bases.push(Macro {
                name: format!("Center_R_U_Back_s{i}_s{j}"),
                moves: get_center3_moves(size, i, j),
                cost: 8,
            });
            center_pure_bases.push(Macro {
                name: format!("Center_R_U_Front_s{i}_s{j}"),
                moves: get_center4_moves(size, i, j),
                cost: 8,
            });
        }
    }
    let pure_center_macros = generate_symmetric_macros(&center_pure_bases, &rotations, size);

    let mut all_center_macros = destructive_center_macros.clone();
    all_center_macros.extend(pure_center_macros.clone());

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

    // Phase 1: Solving Centers
    let mut step = 1;
    let mut global_visited_centers = HashSet::new();
    global_visited_centers.insert(cube.clone());

    loop {
        let misplaced_raw = count_misplaced_centers(cube);
        if misplaced_raw == 0 {
            break;
        }

        // Endgame Lookup (Phase 1 Giai đoạn 2)
        if misplaced_raw <= 8 {
            if let Some(table) = l2c_table {
                if let Some(l2c_moves) = solve_l2c_lookup(cube, table) {
                    cube.apply_moves(&l2c_moves);
                    solved_solution.extend(l2c_moves);
                    break;
                }
            }
        }

        // Determine if we should use coarse or fine search
        let (macros_to_use, bw, depth) = if misplaced_raw > 8 {
            (&all_center_macros, 15, 2)
        } else {
            (&pure_center_macros, 50, 5)
        };

        let mut best_macros = solve_phase_beam_search(
            cube,
            SolverPhase::Phase1Centers,
            macros_to_use,
            bw,
            depth,
            &global_visited_centers,
        );

        if let Some(ref bm) = best_macros {
            if bm.is_empty() {
                // Adaptive Fallback
                best_macros = solve_phase_beam_search(
                    cube,
                    SolverPhase::Phase1Centers,
                    &pure_center_macros,
                    300,
                    8,
                    &global_visited_centers,
                );
            }
        }

        if let Some(bm) = best_macros {
            if bm.is_empty() {
                // Last ditch effort: Short BFS Depth 3
                // (Implementation omitted for now, using existing deep search as fallback)
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
            SolverPhase::Phase2Edges,
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
                    SolverPhase::Phase2Edges,
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
            SolverPhase::Phase3CornersAndParity,
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
                    SolverPhase::Phase3CornersAndParity,
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
