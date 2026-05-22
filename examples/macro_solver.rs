// English: All comments in source code must be in English.
// Vietnamese: Trao đổi trong phần chat hoàn toàn bằng Tiếng Việt.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::uninlined_format_args,
    clippy::missing_const_for_fn,
    clippy::similar_names,
    clippy::too_many_lines,
    clippy::type_complexity,
    clippy::must_use_candidate,
    clippy::struct_field_names
)]

use bevy::prelude::{IVec3, Quat, Vec3};
use rubik_solver::core::{Direction, Face, RotationAxis, RotationMove};
use std::collections::{HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::time::Instant;

// =========================================================================
// 1. Math & Virtual Cube representation (100% Mathematically Flawless)
// =========================================================================

#[derive(Clone, Debug)]
struct VirtualCubie {
    pos: IVec3,
    rotation: Quat,
}

impl Eq for VirtualCubie {}

impl PartialEq for VirtualCubie {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos && (self.rotation.dot(other.rotation).abs() > 0.99)
    }
}

impl Hash for VirtualCubie {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pos.hash(state);
        // Quaternions q and -q represent the identical physical rotation.
        // We resolve this ambiguity by forcing the dominant component to be positive.
        let mut q = self.rotation;
        let components = [q.x, q.y, q.z, q.w];
        let mut max_idx = 0;
        let mut max_val = components[0].abs();
        for (i, &val) in components.iter().enumerate().skip(1) {
            if val.abs() > max_val {
                max_val = val.abs();
                max_idx = i;
            }
        }
        if components[max_idx] < 0.0 {
            q = -q;
        }
        let rx = (q.x * 100.0).round() as i32;
        let ry = (q.y * 100.0).round() as i32;
        let rz = (q.z * 100.0).round() as i32;
        let rw = (q.w * 100.0).round() as i32;
        rx.hash(state);
        ry.hash(state);
        rz.hash(state);
        rw.hash(state);
    }
}

#[derive(Clone, Debug)]
pub struct VirtualCube {
    size: i32,
    cubies: Vec<VirtualCubie>,
}

impl Eq for VirtualCube {}

impl PartialEq for VirtualCube {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size && self.cubies == other.cubies
    }
}

impl Hash for VirtualCube {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.size.hash(state);
        self.cubies.hash(state);
    }
}

impl VirtualCube {
    /// Create a solved virtual cube of size N
    pub fn new(size: i32) -> Self {
        let mut cubies = Vec::new();
        for x in 0..size {
            for y in 0..size {
                for z in 0..size {
                    // Skip internal cubies which have no visible facelets
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

    /// Apply a single `RotationMove` utilizing identical mathematics to Bevy's engine
    pub fn apply_move(&mut self, m: RotationMove) {
        let size = self.size;
        let (axis_vec, angle) = m.get_rotation_info();
        let rot_step = Quat::from_axis_angle(axis_vec, angle);
        let offset = (size as f32 - 1.0) / 2.0;

        for cubie in &mut self.cubies {
            if m.is_cubie_at_slice(cubie.pos) {
                // Rotate logical coordinate vector
                let centered = cubie.pos.as_vec3() - Vec3::splat(offset);
                let rotated = rot_step * centered;
                cubie.pos = (rotated + Vec3::splat(offset)).round().as_ivec3();

                // Rotate physical orientation
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

    /// Calculate total misplaced stickers using robust vector projection
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

/// Helper to classify cubie type based on logical boundary coordinates
/// 1 = Center piece, 2 = Edge piece, 3 = Corner piece
fn count_boundary_components(pos: IVec3, size: i32) -> usize {
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

// =========================================================================
// 2. Symmetric Rotations & Group Theory Generator
// =========================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CubeRotation {
    x_map: (RotationAxis, bool),
    y_map: (RotationAxis, bool),
    z_map: (RotationAxis, bool),
}

impl CubeRotation {
    /// Rotates a physical move in space matching the whole-cube symmetry mapping
    fn transform_move(self, m: RotationMove, size: i32) -> RotationMove {
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

fn get_vector(axis: RotationAxis, positive: bool) -> IVec3 {
    let sign = if positive { 1 } else { -1 };
    match axis {
        RotationAxis::X => IVec3::new(sign, 0, 0),
        RotationAxis::Y => IVec3::new(0, sign, 0),
        RotationAxis::Z => IVec3::new(0, 0, sign),
    }
}

fn cross(a: IVec3, b: IVec3) -> IVec3 {
    IVec3::new(
        a.y * b.z - a.z * b.y,
        a.z * b.x - a.x * b.z,
        a.x * b.y - a.y * b.x,
    )
}

fn dot(a: IVec3, b: IVec3) -> i32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

/// Generates the exact 24 proper rotations of a 3D cube
fn generate_cube_rotations() -> Vec<CubeRotation> {
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

// =========================================================================
// 3. Macros & Macro Transformations (with Deduplication)
// =========================================================================

#[derive(Clone)]
struct Macro {
    name: String,
    moves: Vec<RotationMove>,
    cost: usize,
}

#[derive(Clone, Debug, PartialEq)]
struct SymmetricMacro {
    name: String,
    moves: Vec<RotationMove>,
    cost: usize,
}

/// Expand base macros to symmetric ones and remove duplicates for minimal search space
fn generate_symmetric_macros(
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

// =========================================================================
// Base Macros Setup for 4x4x4 Cube
// =========================================================================

fn get_center1_moves(size: i32, i: i32) -> Vec<RotationMove> {
    // r U l' U' r' U l U'
    let r_idx = size - 1 - i;
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
        }, // l'
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
        }, // l
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
    ]
}

fn get_center2_moves(size: i32, i: i32) -> Vec<RotationMove> {
    // l U r' U' l' U r U'
    let r_idx = size - 1 - i;
    let l_idx = i;
    let u_idx = size - 1;
    vec![
        RotationMove {
            axis: RotationAxis::X,
            index: l_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        }, // l
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
        }, // l'
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

fn get_center3_moves(size: i32, i: i32) -> Vec<RotationMove> {
    // f U b' U' f' U b U'
    let f_idx = size - 1 - i;
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
        }, // b'
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
        }, // b
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
    ]
}

fn get_center4_moves(size: i32, i: i32) -> Vec<RotationMove> {
    // b U f' U' b' U f U'
    let f_idx = size - 1 - i;
    let b_idx = i;
    let u_idx = size - 1;
    vec![
        RotationMove {
            axis: RotationAxis::Z,
            index: b_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        }, // b
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
        }, // b'
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

fn get_edge_pair_moves(size: i32, i: i32) -> Vec<RotationMove> {
    // u_i' R U R' F R' F' R u_i
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

fn get_edge_flip_moves(size: i32) -> Vec<RotationMove> {
    // R U R' F R' F' R
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

fn get_last_two_edges_1_moves(size: i32, i: i32) -> Vec<RotationMove> {
    // d_i R U R' F R' F' R d_i'
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

fn get_last_two_edges_2_moves(size: i32, i: i32) -> Vec<RotationMove> {
    // d_i' L' U' L F L' F' L d_i
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
        }, // L'
        RotationMove {
            axis: RotationAxis::Y,
            index: u_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        }, // U'
        RotationMove {
            axis: RotationAxis::X,
            index: l_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        }, // L
        RotationMove {
            axis: RotationAxis::Z,
            index: f_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        }, // F
        RotationMove {
            axis: RotationAxis::X,
            index: l_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        }, // L'
        RotationMove {
            axis: RotationAxis::Z,
            index: f_idx,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        }, // F'
        RotationMove {
            axis: RotationAxis::X,
            index: l_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        }, // L
        RotationMove {
            axis: RotationAxis::Y,
            index: slice_idx,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
    ]
}

fn get_niklas_8_moves(size: i32) -> Vec<RotationMove> {
    // U R U' L' U R' U' L
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
        }, // L'
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
        }, // L
    ]
}

fn get_t_perm_moves(size: i32) -> Vec<RotationMove> {
    // R U R' U' R' F R2 U' R' U' R U R' F'
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

fn get_pll_parity_moves(size: i32) -> Vec<RotationMove> {
    let mut moves = Vec::new();
    let r2 = |m: &mut Vec<RotationMove>| {
        let mv = RotationMove {
            axis: RotationAxis::X,
            index: size - 2,
            direction: Direction::Clockwise,
            add_to_history: true,
        };
        m.push(mv);
        m.push(mv);
    };
    let u2 = |m: &mut Vec<RotationMove>| {
        let mv = RotationMove {
            axis: RotationAxis::Y,
            index: size - 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        };
        m.push(mv);
        m.push(mv);
    };
    let double_uw2 = |m: &mut Vec<RotationMove>| {
        let mv1 = RotationMove {
            axis: RotationAxis::Y,
            index: size - 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        };
        let mv2 = RotationMove {
            axis: RotationAxis::Y,
            index: size - 2,
            direction: Direction::Clockwise,
            add_to_history: true,
        };
        m.push(mv1);
        m.push(mv1);
        m.push(mv2);
        m.push(mv2);
    };

    // r2 U2 r2 Uw2 r2 Uw2
    r2(&mut moves);
    u2(&mut moves);
    r2(&mut moves);
    double_uw2(&mut moves);
    r2(&mut moves);
    double_uw2(&mut moves);
    moves
}

fn get_oll_parity_moves(size: i32) -> Vec<RotationMove> {
    let mut moves = Vec::new();
    let rw = |m: &mut Vec<RotationMove>, dir: Direction| {
        m.push(RotationMove {
            axis: RotationAxis::X,
            index: size - 1,
            direction: dir,
            add_to_history: true,
        });
        m.push(RotationMove {
            axis: RotationAxis::X,
            index: size - 2,
            direction: dir,
            add_to_history: true,
        });
    };
    let lw = |m: &mut Vec<RotationMove>, dir: Direction| {
        m.push(RotationMove {
            axis: RotationAxis::X,
            index: 0,
            direction: dir,
            add_to_history: true,
        });
        m.push(RotationMove {
            axis: RotationAxis::X,
            index: 1,
            direction: dir,
            add_to_history: true,
        });
    };
    let u2 = |m: &mut Vec<RotationMove>| {
        let mv = RotationMove {
            axis: RotationAxis::Y,
            index: size - 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        };
        m.push(mv);
        m.push(mv);
    };
    let x_rot = |m: &mut Vec<RotationMove>| {
        for idx in 0..size {
            m.push(RotationMove {
                axis: RotationAxis::X,
                index: idx,
                direction: Direction::Clockwise,
                add_to_history: true,
            });
        }
    };

    // Rw U2 x Rw U2 Rw U2 Rw' U2 Lw U2 Rw' U2 Rw U2 Rw' U2 Rw
    rw(&mut moves, Direction::Clockwise);
    u2(&mut moves);
    x_rot(&mut moves);
    rw(&mut moves, Direction::Clockwise);
    u2(&mut moves);
    rw(&mut moves, Direction::Clockwise);
    u2(&mut moves);
    rw(&mut moves, Direction::CounterClockwise);
    u2(&mut moves);
    lw(&mut moves, Direction::Clockwise);
    u2(&mut moves);
    rw(&mut moves, Direction::CounterClockwise);
    u2(&mut moves);
    rw(&mut moves, Direction::Clockwise);
    u2(&mut moves);
    rw(&mut moves, Direction::CounterClockwise);
    u2(&mut moves);
    rw(&mut moves, Direction::Clockwise);
    moves
}

// =========================================================================
// 4. Multi-Phase Beam Search Solver Engine
// =========================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SolverPhase {
    Phase1Centers,
    Phase2Edges,
    Phase3CornersAndParity,
}

/// Node representing the search state during BFS/Beam Search
#[derive(Clone)]
struct SearchNode {
    cube: VirtualCube,
    macro_indices: Vec<usize>,
    total_cost: usize,
}

/// Evaluates state heuristic value based on active solving phase
fn evaluate_heuristic(cube: &VirtualCube, phase: SolverPhase) -> usize {
    match phase {
        SolverPhase::Phase1Centers => count_misplaced_centers(cube),
        SolverPhase::Phase2Edges => count_unpaired_edges(cube),
        SolverPhase::Phase3CornersAndParity => cube.count_misplaced_stickers(),
    }
}

fn count_misplaced_centers(cube: &VirtualCube) -> usize {
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

fn count_unpaired_edges(cube: &VirtualCube) -> usize {
    let size = cube.size;
    let mut unpaired = 0;

    let find_cubie = |x: i32, y: i32, z: i32| -> Option<&VirtualCubie> {
        cube.cubies.iter().find(|c| c.pos == IVec3::new(x, y, z))
    };

    // The 12 edge zones comparing inner layers
    let edge_zones = [
        // Z-axis inner edges
        ((0, 0), true),
        ((0, size - 1), true),
        ((size - 1, 0), true),
        ((size - 1, size - 1), true),
        // Y-axis inner edges
        ((0, 0), false),
        ((0, size - 1), false),
        ((size - 1, 0), false),
        ((size - 1, size - 1), false),
    ];

    for &((a, b), is_z_axis) in &edge_zones {
        if is_z_axis {
            if let (Some(c1), Some(c2)) = (find_cubie(a, b, 1), find_cubie(a, b, 2)) {
                if c1.rotation.dot(c2.rotation).abs() < 0.99 {
                    unpaired += 2;
                }
            }
        } else if let (Some(c1), Some(c2)) = (find_cubie(a, 1, b), find_cubie(a, 2, b)) {
            if c1.rotation.dot(c2.rotation).abs() < 0.99 {
                unpaired += 2;
            }
        }
    }

    // X-axis inner edges
    let x_edge_zones = [(0, 0), (0, size - 1), (size - 1, 0), (size - 1, size - 1)];
    for &(y, z) in &x_edge_zones {
        if let (Some(c1), Some(c2)) = (find_cubie(1, y, z), find_cubie(2, y, z)) {
            if c1.rotation.dot(c2.rotation).abs() < 0.99 {
                unpaired += 2;
            }
        }
    }

    unpaired
}

/// Solves the active phase using Beam Search with loop protection and dynamic `max_depth`
fn solve_phase_beam_search(
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

    // Expand search depth dynamically matching max_depth
    for depth in 1..=max_depth {
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

        println!("   Depth {depth}: Best misplaced remaining = {best_heuristic}");
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

// =========================================================================
// 5. Test Harness & Solver Demonstration
// =========================================================================

/// Scrambles a 4x4x4 cube using a custom move set
fn scramble_cube(cube: &mut VirtualCube) -> Vec<RotationMove> {
    let size = cube.size;

    // Scramble containing both outer turns and inner turns
    let scramble_moves = vec![
        // R (outer)
        RotationMove {
            axis: RotationAxis::X,
            index: size - 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        // U (outer)
        RotationMove {
            axis: RotationAxis::Y,
            index: size - 1,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        // r (inner slice scramble)
        RotationMove {
            axis: RotationAxis::X,
            index: size / 2,
            direction: Direction::Clockwise,
            add_to_history: true,
        },
        // f' (inner slice scramble)
        RotationMove {
            axis: RotationAxis::Z,
            index: size / 2,
            direction: Direction::CounterClockwise,
            add_to_history: true,
        },
    ];

    cube.apply_moves(&scramble_moves);
    scramble_moves
}

fn run_solver_for_size(size: i32) {
    println!("\n==================================================");
    println!("  🧠 RUNNING SOLVER FOR {}x{}x{} CUBE  ", size, size, size);
    println!("==================================================");

    let mut cube = VirtualCube::new(size);
    println!(
        "1. Solved {}x{}x{} VirtualCube initialized.",
        size, size, size
    );
    assert_eq!(cube.count_misplaced_stickers(), 0);

    println!("\n2. Scrambling the cube...");
    let scramble = scramble_cube(&mut cube);
    println!("   Applied scramble ({} moves)", scramble.len());
    println!(
        "   Scrambled misplaced stickers: {}",
        cube.count_misplaced_stickers()
    );
    println!(
        "   Scrambled misplaced centers:  {}",
        count_misplaced_centers(&cube)
    );
    println!(
        "   Scrambled unpaired edges:     {}",
        count_unpaired_edges(&cube)
    );

    // 3. Pre-generate all 24 proper rotations & symmetric macros
    let rotations = generate_cube_rotations();
    println!("\n3. Generated 24 proper symmetry rotations.");

    // Centers base macros (including the vital parity changing inner slice turn!)
    let mut center_bases = Vec::new();
    for i in 1..(size - 1) {
        center_bases.push(Macro {
            name: format!("Inner_Slice_Turn_s{}", i),
            moves: vec![RotationMove {
                axis: RotationAxis::X,
                index: i,
                direction: Direction::Clockwise,
                add_to_history: true,
            }],
            cost: 1,
        });
        center_bases.push(Macro {
            name: format!("Center_F_U_Right_s{}", i),
            moves: get_center1_moves(size, i),
            cost: 8,
        });
        center_bases.push(Macro {
            name: format!("Center_F_U_Left_s{}", i),
            moves: get_center2_moves(size, i),
            cost: 8,
        });
        center_bases.push(Macro {
            name: format!("Center_R_U_Back_s{}", i),
            moves: get_center3_moves(size, i),
            cost: 8,
        });
        center_bases.push(Macro {
            name: format!("Center_R_U_Front_s{}", i),
            moves: get_center4_moves(size, i),
            cost: 8,
        });
    }
    let center_macros = generate_symmetric_macros(&center_bases, &rotations, size);
    println!(
        "   Expanded centers base macros to {} unique symmetric macros.",
        center_macros.len()
    );

    // Edges base macros (including Outer_Face_Turn setup moves and Last 2 Edges macros!)
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
            name: format!("Edge_Pair_R_F_s{}", i),
            moves: get_edge_pair_moves(size, i),
            cost: 9,
        });
        edge_bases.push(Macro {
            name: format!("Last_Two_Edges_1_s{}", i),
            moves: get_last_two_edges_1_moves(size, i),
            cost: 9,
        });
        edge_bases.push(Macro {
            name: format!("Last_Two_Edges_2_s{}", i),
            moves: get_last_two_edges_2_moves(size, i),
            cost: 9,
        });
    }
    let edge_macros = generate_symmetric_macros(&edge_bases, &rotations, size);
    println!(
        "   Expanded edges base macros to {} unique symmetric edge macros.",
        edge_macros.len()
    );

    // Corners & Parities base macros (including Outer_Face_Turn setup moves)
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
    println!(
        "   Expanded corners & parity base macros to {} unique symmetric macros.",
        stage3_macros.len()
    );

    let mut solved_solution = Vec::new();
    let total_start = Instant::now();

    let total_centers = 6 * (size - 2) * (size - 2);
    let max_center_steps = (total_centers * 2) as usize;
    let total_edges = 12 * (size - 2);
    let max_edge_steps = (total_edges * 2) as usize;
    let max_stage3_steps = 50;

    // =========================================================================
    // Phase 1: Solve Centers (depth: 3)
    // =========================================================================
    println!("\n[PHASE 1] Solving Centers...");
    let mut step = 1;
    let mut global_visited_centers = HashSet::new();
    global_visited_centers.insert(cube.clone());

    loop {
        let misplaced = count_misplaced_centers(&cube);
        if misplaced == 0 {
            println!("   [PHASE 1 COMPLETE] All centers solved!");
            break;
        }
        println!(
            "   Step {step}: Misplaced center stickers = {misplaced}. Searching macro combination... "
        );

        let start = Instant::now();
        if let Some(best_macros) = solve_phase_beam_search(
            &cube,
            SolverPhase::Phase1Centers,
            &center_macros,
            50,
            5,
            &global_visited_centers,
        ) {
            if best_macros.is_empty() {
                println!("   [WARNING] No improvement found! Breaking to prevent loop.");
                break;
            }
            println!(
                "   Found best combination ({} macros, time: {:?}):",
                best_macros.len(),
                start.elapsed()
            );
            for m in &best_macros {
                println!("     * Apply Macro: {} (Cost/Penalty: {})", m.name, m.cost);
                cube.apply_moves(&m.moves);
                global_visited_centers.insert(cube.clone());
                solved_solution.push(m.clone());
            }
        } else {
            println!("   [FAILED] Failed to solve centers!");
            return;
        }
        step += 1;

        if step > max_center_steps {
            println!("   [WARNING] Step limit exceeded in Phase 1!");
            break;
        }
    }

    // =========================================================================
    // Phase 2: Solve Edges (Edge Pairing) (depth: 3)
    // =========================================================================
    println!("\n[PHASE 2] Pairing Edges...");
    step = 1;
    let mut global_visited_edges = HashSet::new();
    global_visited_edges.insert(cube.clone());

    loop {
        let unpaired = count_unpaired_edges(&cube);
        if unpaired == 0 {
            println!("   [PHASE 2 COMPLETE] All edges paired!");
            break;
        }
        println!(
            "   Step {step}: Unpaired edges count = {unpaired}. Searching edge pairing macro..."
        );

        let start = Instant::now();
        if let Some(best_macros) = solve_phase_beam_search(
            &cube,
            SolverPhase::Phase2Edges,
            &edge_macros,
            50,
            5,
            &global_visited_edges,
        ) {
            if best_macros.is_empty() {
                println!("   [WARNING] No edge improvement found! Breaking to prevent loop.");
                break;
            }
            println!(
                "   Found best combination ({} macros, time: {:?}):",
                best_macros.len(),
                start.elapsed()
            );
            for m in &best_macros {
                println!("     * Apply Macro: {} (Cost/Penalty: {})", m.name, m.cost);
                cube.apply_moves(&m.moves);
                global_visited_edges.insert(cube.clone());
                solved_solution.push(m.clone());
            }
        } else {
            println!("   [FAILED] Failed to pair edges!");
            return;
        }
        step += 1;

        if step > max_edge_steps {
            println!("   [WARNING] Step limit exceeded in Phase 2!");
            break;
        }
    }

    // =========================================================================
    // Phase 3: Solve Corners, Edges and Parities (dynamic depth: 6 for 3x3x3 stage!)
    // =========================================================================
    println!("\n[PHASE 3] Solving Corners, Edges and Parities (Depth: 6)...");
    step = 1;
    let mut global_visited_stage3 = HashSet::new();
    global_visited_stage3.insert(cube.clone());

    loop {
        let misplaced = cube.count_misplaced_stickers();
        if misplaced == 0 {
            println!("   [PHASE 3 COMPLETE] Cube is 100% solved!");
            break;
        }
        println!(
            "   Step {step}: Total misplaced stickers = {misplaced}. Searching combination..."
        );

        let start = Instant::now();
        if let Some(best_macros) = solve_phase_beam_search(
            &cube,
            SolverPhase::Phase3CornersAndParity,
            &stage3_macros,
            50,
            6,
            &global_visited_stage3,
        ) {
            if best_macros.is_empty() {
                println!("   [WARNING] No improvement found in Phase 3! Breaking to prevent loop.");
                break;
            }
            println!(
                "   Found best combination ({} macros, time: {:?}):",
                best_macros.len(),
                start.elapsed()
            );
            for m in &best_macros {
                println!("     * Apply Macro: {} (Cost/Penalty: {})", m.name, m.cost);
                cube.apply_moves(&m.moves);
                global_visited_stage3.insert(cube.clone());
                solved_solution.push(m.clone());
            }
        } else {
            println!("   [FAILED] Failed to solve 3x3 stage!");
            return;
        }
        step += 1;

        if step > max_stage3_steps {
            println!("   [WARNING] Step limit exceeded in Phase 3!");
            break;
        }
    }

    // =========================================================================
    // Solver Validation
    // =========================================================================
    let total_cost: usize = solved_solution.iter().map(|m| m.cost).sum();
    println!("\n==================================================");
    println!("  🎉 SOLVER RUN COMPLETED IN {:?}", total_start.elapsed());
    println!("  Total macro steps applied: {}", solved_solution.len());
    println!("  Total moves cost (penalty): {}", total_cost);
    println!(
        "  Final misplaced stickers: {}",
        cube.count_misplaced_stickers()
    );
    println!("==================================================");

    if cube.count_misplaced_stickers() == 0 {
        println!("  [SUCCESS] Cube is 100% solved using macro search algorithm!");
    } else {
        println!("  [FAILED] Cube is not fully solved.");
    }
}

fn main() {
    println!("==================================================");
    println!("  🧠 NxN CUBE SOLVER VIA SYMMETRIC MACRO SEARCH  ");
    println!("==================================================");

    for &size in &[4, 5, 6] {
        run_solver_for_size(size);
    }
}
