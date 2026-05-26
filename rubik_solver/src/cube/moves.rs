// Implementation of cube rotation operations using a consistent 3D coordinate mapping.
// All comments in source files must be in English.

use crate::cube::{Cube, CubeError, Face};

/// Helper to convert 2D face coordinates to 3D spatial coordinates.
/// 3D coordinates:
/// - X-axis: Left (0) to Right (n-1). L is at x=0, R is at x=n-1.
/// - Y-axis: Down (0) to Up (n-1). D is at y=0, U is at y=n-1.
/// - Z-axis: Back (0) to Front (n-1). B is at z=0, F is at z=n-1.
#[inline]
pub fn to_3d(face: Face, r: usize, c: usize, n: usize) -> (usize, usize, usize) {
    let n_1 = n - 1;
    match face {
        Face::U => (c, n_1, r),
        Face::D => (c, 0, n_1 - r),
        Face::F => (c, n_1 - r, n_1),
        Face::B => (n_1 - c, n_1 - r, 0),
        Face::L => (0, n_1 - r, c),
        Face::R => (n_1, n_1 - r, n_1 - c),
    }
}

/// Helper to convert 3D spatial coordinates back to 2D face coordinates.
#[inline]
pub fn from_3d_to_rc(face: Face, x: usize, y: usize, z: usize, n: usize) -> (usize, usize) {
    let n_1 = n - 1;
    match face {
        Face::U => (z, x),
        Face::D => (n_1 - z, x),
        Face::F => (n_1 - y, x),
        Face::B => (n_1 - y, n_1 - x),
        Face::L => (n_1 - y, z),
        Face::R => (n_1 - y, n_1 - z),
    }
}

/// Axis of rotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X, // Left-Right (L/R)
    Y, // Down-Up (D/U)
    Z, // Back-Front (B/F)
}

impl Cube {
    /// Applies a rotation to a specific slice along an axis.
    /// - `axis`: The axis of rotation (X, Y, or Z).
    /// - `slice_idx`: The index of the slice (0 to n-1) along the axis.
    /// - `clockwise`: Hướng xoay (CW or CCW) from the positive direction of the axis.
    ///   Positive directions: R (+X), U (+Y), F (+Z).
    pub fn rotate_slice(
        &mut self,
        axis: Axis,
        slice_idx: usize,
        clockwise: bool,
    ) -> Result<(), CubeError> {
        if slice_idx >= self.size {
            return Err(CubeError::IndexOutOfBounds(slice_idx, self.size));
        }
        let n = self.size;
        let mut new_grid = self.grid.clone();

        for face_idx in 0..6 {
            let face = match face_idx {
                0 => Face::U,
                1 => Face::D,
                2 => Face::F,
                3 => Face::B,
                4 => Face::L,
                _ => Face::R,
            };

            for r in 0..n {
                for c in 0..n {
                    let (x, y, z) = to_3d(face, r, c, n);

                    // Check if this facelet lies in the rotating slice
                    let in_slice = match axis {
                        Axis::X => x == slice_idx,
                        Axis::Y => y == slice_idx,
                        Axis::Z => z == slice_idx,
                    };

                    if in_slice {
                        // Apply reverse rotation to find source coordinates (x_old, y_old, z_old)
                        let (x_old, y_old, z_old) = match (axis, clockwise) {
                            // Rotate around Y-axis (Up/Down)
                            // CW: (x, y, z) -> (n-1-z, y, x). CCW: (x, y, z) -> (z, y, n-1-x).
                            // For source (reverse rotation):
                            (Axis::Y, true) => (z, y, n - 1 - x),
                            (Axis::Y, false) => (n - 1 - z, y, x),

                            // Rotate around X-axis (Right/Left)
                            // CW: (x, y, z) -> (x, z, n-1-y). CCW: (x, y, z) -> (x, n-1-z, y).
                            (Axis::X, true) => (x, z, n - 1 - y),
                            (Axis::X, false) => (x, n - 1 - z, y),

                            // Rotate around Z-axis (Front/Back)
                            // CW: (x, y, z) -> (n-1-y, x, z). CCW: (x, y, z) -> (y, n-1-x, z).
                            (Axis::Z, true) => (n - 1 - y, x, z),
                            (Axis::Z, false) => (y, n - 1 - x, z),
                        };

                        // Find the source face (face_old) by applying reverse transformation on face normals
                        let face_old = match (axis, clockwise) {
                            (Axis::Y, true) => match face {
                                Face::U => Face::U,
                                Face::D => Face::D,
                                Face::F => Face::R,
                                Face::R => Face::B,
                                Face::B => Face::L,
                                Face::L => Face::F,
                            },
                            (Axis::Y, false) => match face {
                                Face::U => Face::U,
                                Face::D => Face::D,
                                Face::F => Face::L,
                                Face::L => Face::B,
                                Face::B => Face::R,
                                Face::R => Face::F,
                            },
                            (Axis::X, true) => match face {
                                Face::R => Face::R,
                                Face::L => Face::L,
                                Face::U => Face::F,
                                Face::F => Face::D,
                                Face::D => Face::B,
                                Face::B => Face::U,
                            },
                            (Axis::X, false) => match face {
                                Face::R => Face::R,
                                Face::L => Face::L,
                                Face::U => Face::B,
                                Face::B => Face::D,
                                Face::D => Face::F,
                                Face::F => Face::U,
                            },
                            (Axis::Z, true) => match face {
                                Face::F => Face::F,
                                Face::B => Face::B,
                                Face::U => Face::L,
                                Face::L => Face::D,
                                Face::D => Face::R,
                                Face::R => Face::U,
                            },
                            (Axis::Z, false) => match face {
                                Face::F => Face::F,
                                Face::B => Face::B,
                                Face::U => Face::R,
                                Face::R => Face::D,
                                Face::D => Face::L,
                                Face::L => Face::U,
                            },
                        };

                        let (r_old, c_old) = from_3d_to_rc(face_old, x_old, y_old, z_old, n);
                        let source_val = self.get(face_old, r_old, c_old)?;

                        let idx = (face as usize) * n * n + r * n + c;
                        if let Some(elem) = new_grid.get_mut(idx) {
                            *elem = source_val;
                        }
                    }
                }
            }
        }

        self.grid = new_grid;
        Ok(())
    }

    /// Parses and applies a move string like "U", "R'", "u1", "r2", etc.
    /// Standard moves:
    /// - Outer faces: U, D, F, B, L, R (with optional ' for CCW, or 2 for double turn).
    /// - Inner slices:
    ///   - u{i}, d{i}, f{i}, b{i}, l{i}, r{i} where i is 1-indexed slice number from 1 to n-2.
    ///   - Example: "u1" is the first slice below U. "r2" is the second slice to the left of R.
    pub fn apply_move(&mut self, move_str: &str) -> Result<(), CubeError> {
        if move_str.is_empty() {
            return Err(CubeError::InvalidMove(move_str.to_string()));
        }

        let mut base_str = move_str;
        let mut double = false;
        let mut clockwise = true;

        if move_str.ends_with('\'') {
            clockwise = false;
            base_str = &move_str[..move_str.len() - 1];
        } else if move_str.ends_with('2') {
            let first = move_str.chars().next().unwrap_or(' ');
            let is_outer = first.is_uppercase();
            let is_slice_double = first.is_lowercase()
                && move_str.len() >= 3
                && move_str
                    .chars()
                    .nth(move_str.len() - 2)
                    .unwrap_or(' ')
                    .is_ascii_digit();

            if is_outer || is_slice_double {
                double = true;
                base_str = &move_str[..move_str.len() - 1];
            }
        }

        let mut chars = base_str.chars();
        let first_char = chars
            .next()
            .ok_or_else(|| CubeError::InvalidMove(move_str.to_string()))?;

        let mut slice_num: Option<usize> = None;
        let move_type = first_char;

        // Parse slice index if inner move (e.g., u1, r2)
        if first_char.is_lowercase() {
            let mut num_str = String::new();
            for c in chars {
                if c.is_ascii_digit() {
                    num_str.push(c);
                } else {
                    return Err(CubeError::InvalidMove(move_str.to_string()));
                }
            }
            let val = num_str
                .parse::<usize>()
                .map_err(|_| CubeError::InvalidMove(move_str.to_string()))?;
            if val == 0 || val >= self.size - 1 {
                return Err(CubeError::InvalidMove(move_str.to_string()));
            }
            slice_num = Some(val);
        }

        let n = self.size;
        let (axis, slice_idx, axis_cw) = match move_type {
            // Outer Face Moves
            'U' => (Axis::Y, n - 1, clockwise),
            'D' => (Axis::Y, 0, !clockwise), // D direction is opposite to U
            'R' => (Axis::X, n - 1, clockwise),
            'L' => (Axis::X, 0, !clockwise), // L direction is opposite to R
            'F' => (Axis::Z, n - 1, clockwise),
            'B' => (Axis::Z, 0, !clockwise), // B direction is opposite to F

            // Inner Slice Moves
            'u' => {
                let idx = slice_num.ok_or_else(|| CubeError::InvalidMove(move_str.to_string()))?;
                (Axis::Y, n - 1 - idx, clockwise)
            }
            'd' => {
                let idx = slice_num.ok_or_else(|| CubeError::InvalidMove(move_str.to_string()))?;
                (Axis::Y, idx, !clockwise)
            }
            'r' => {
                let idx = slice_num.ok_or_else(|| CubeError::InvalidMove(move_str.to_string()))?;
                (Axis::X, n - 1 - idx, clockwise)
            }
            'l' => {
                let idx = slice_num.ok_or_else(|| CubeError::InvalidMove(move_str.to_string()))?;
                (Axis::X, idx, !clockwise)
            }
            'f' => {
                let idx = slice_num.ok_or_else(|| CubeError::InvalidMove(move_str.to_string()))?;
                (Axis::Z, n - 1 - idx, clockwise)
            }
            'b' => {
                let idx = slice_num.ok_or_else(|| CubeError::InvalidMove(move_str.to_string()))?;
                (Axis::Z, idx, !clockwise)
            }
            _ => return Err(CubeError::InvalidMove(move_str.to_string())),
        };

        if double {
            self.rotate_slice(axis, slice_idx, axis_cw)?;
            self.rotate_slice(axis, slice_idx, axis_cw)?;
        } else {
            self.rotate_slice(axis, slice_idx, axis_cw)?;
        }

        Ok(())
    }
}
