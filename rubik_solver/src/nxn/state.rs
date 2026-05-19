#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::doc_markdown,
    clippy::missing_const_for_fn
)]

use crate::core::{Face, RotationAxis, RotationMove};
use bevy::prelude::*;

/// Order of faces stored in standard representations
pub const FACES_ORDER: [Face; 6] = [
    Face::Up,
    Face::Right,
    Face::Front,
    Face::Down,
    Face::Left,
    Face::Back,
];

/// A single facelet (sticker) containing its coordinate, current face normal orientation, and color.
#[derive(Clone, Debug)]
pub struct Facelet {
    /// Logical grid coordinate of the cubie (0..size)
    pub coord: IVec3,
    /// Current logical face orientation
    pub face: Face,
    /// Logical color of the facelet
    pub color: Face,
}

/// Logical state of an NxN Rubik's Cube
#[derive(Clone, Debug)]
pub struct NxNState {
    pub size: usize,
    pub facelets: Vec<Facelet>,
}

impl NxNState {
    /// Create a solved NxN cube state
    pub fn new(size: usize) -> Self {
        let mut facelets = Vec::new();
        for &face in &FACES_ORDER {
            for row in 0..size {
                for col in 0..size {
                    if let Some(coord) = Self::get_logical_coord(face, row, col, size) {
                        facelets.push(Facelet {
                            coord,
                            face,
                            color: face,
                        });
                    }
                }
            }
        }
        Self { size, facelets }
    }

    /// Calculate the logical IVec3 grid coordinate (0..size) for a facelet
    pub fn get_logical_coord(face: Face, row: usize, col: usize, size: usize) -> Option<IVec3> {
        let s = size as i32;
        let r = row as i32;
        let c = col as i32;

        match face {
            Face::Up => Some(IVec3::new(c, s - 1, r)),
            Face::Down => Some(IVec3::new(c, 0, s - 1 - r)),
            Face::Right => Some(IVec3::new(s - 1, s - 1 - r, s - 1 - c)),
            Face::Left => Some(IVec3::new(0, s - 1 - r, c)),
            Face::Front => Some(IVec3::new(c, s - 1 - r, s - 1)),
            Face::Back => Some(IVec3::new(s - 1 - c, s - 1 - r, 0)),
        }
    }

    /// Scrapes state from Bevy entities
    pub fn from_bevy(
        size: usize,
        faces: &Query<(&crate::core::CubieFace, &GlobalTransform)>,
        cube_transform: &GlobalTransform,
        mapping: crate::core::FaceMapping,
    ) -> Option<Self> {
        let mut facelets = Vec::new();
        let size_f32 = size as f32;
        let step = 3.0 / size_f32;
        let half_size = size_f32 / 2.0;

        for &logic_face in &FACES_ORDER {
            let (phys_face, right_vec, down_vec) = mapping.get_face_config(logic_face);
            let normal = phys_face.normal();

            for row in 0..size {
                for col in 0..size {
                    #[allow(clippy::cast_precision_loss)]
                    let i = (col as f32 + 0.5 - half_size) * step;
                    #[allow(clippy::cast_precision_loss)]
                    let j = (row as f32 + 0.5 - half_size) * step;
                    let target_pos = normal * 1.5 + right_vec * i + down_vec * j;

                    let color = crate::helpers::find_facelet_color_at(
                        target_pos,
                        normal,
                        faces,
                        cube_transform,
                    )?;

                    let logic_color = mapping.get_logic_face_for_physical(color);
                    if let Some(coord) = Self::get_logical_coord(logic_face, row, col, size) {
                        facelets.push(Facelet {
                            coord,
                            face: logic_face,
                            color: logic_color,
                        });
                    }
                }
            }
        }

        Some(Self { size, facelets })
    }

    /// Generate the flat 2D representation for 3x3 solver compatibility
    /// or for printing / debugging. Returns a string of 6 * size * size chars.
    pub fn to_string_rep(&self) -> String {
        let mut result = vec![' '; 6 * self.size * self.size];
        for (face_idx, &face) in FACES_ORDER.iter().enumerate() {
            for row in 0..self.size {
                for col in 0..self.size {
                    if let Some(coord) = Self::get_logical_coord(face, row, col, self.size) {
                        if let Some(facelet) = self
                            .facelets
                            .iter()
                            .find(|f| f.coord == coord && f.face == face)
                        {
                            let ch = match facelet.color {
                                Face::Up => 'U',
                                Face::Right => 'R',
                                Face::Front => 'F',
                                Face::Down => 'D',
                                Face::Left => 'L',
                                Face::Back => 'B',
                            };
                            result[face_idx * self.size * self.size + row * self.size + col] = ch;
                        }
                    }
                }
            }
        }
        result.into_iter().collect()
    }

    /// Apply a single RotationMove to our virtual state
    pub fn apply_move(&mut self, m: RotationMove) {
        let (axis_vec, angle) = m.get_rotation_info();
        let size_i32 = self.size as i32;

        // Perform rotation on all matching facelets
        for facelet in &mut self.facelets {
            let is_matched = match m.axis {
                RotationAxis::X => facelet.coord.x == m.index,
                RotationAxis::Y => facelet.coord.y == m.index,
                RotationAxis::Z => facelet.coord.z == m.index,
            };

            if is_matched {
                // Rotate coordinates
                let offset = (size_i32 as f32 - 1.0) / 2.0;
                let rotation = Quat::from_axis_angle(axis_vec, angle);
                let centered = facelet.coord.as_vec3() - Vec3::splat(offset);
                let rotated = rotation * centered;
                let restored = rotated + Vec3::splat(offset);
                facelet.coord = restored.round().as_ivec3();

                // Rotate face normal orientation
                let normal = facelet.face.normal();
                let rotated_normal = rotation * normal;
                if let Some(new_face) = Face::from_normal(rotated_normal) {
                    facelet.face = new_face;
                }
            }
        }
    }

    /// Apply a slice move with multiple moves
    pub fn apply_moves(&mut self, moves: &[RotationMove]) {
        for &m in moves {
            self.apply_move(m);
        }
    }
}
