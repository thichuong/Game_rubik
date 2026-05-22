use bevy::prelude::*;

use serde::{Deserialize, Serialize};

/// Enum representing the 6 faces of a cube.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Face {
    Up,    // +Y (White)
    Down,  // -Y (Yellow)
    Right, // +X (Red)
    Left,  // -X (Orange)
    Front, // +Z (Green)
    Back,  // -Z (Blue)
}

impl Face {
    /// Get the normal vector for the face.
    pub const fn normal(self) -> Vec3 {
        match self {
            Self::Up => Vec3::Y,
            Self::Down => Vec3::NEG_Y,
            Self::Right => Vec3::X,
            Self::Left => Vec3::NEG_X,
            Self::Front => Vec3::Z,
            Self::Back => Vec3::NEG_Z,
        }
    }

    /// Find the Face corresponding to a normal vector.
    pub fn from_normal(normal: Vec3) -> Option<Self> {
        let n = normal.round();
        if n.distance(Vec3::Y) < 0.1 {
            Some(Self::Up)
        } else if n.distance(Vec3::NEG_Y) < 0.1 {
            Some(Self::Down)
        } else if n.distance(Vec3::X) < 0.1 {
            Some(Self::Right)
        } else if n.distance(Vec3::NEG_X) < 0.1 {
            Some(Self::Left)
        } else if n.distance(Vec3::Z) < 0.1 {
            Some(Self::Front)
        } else if n.distance(Vec3::NEG_Z) < 0.1 {
            Some(Self::Back)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RotationAxis {
    X,
    Y,
    Z,
}

impl RotationAxis {
    /// Get the unit vector for the rotation axis.
    pub const fn vector(self) -> Vec3 {
        match self {
            Self::X => Vec3::X,
            Self::Y => Vec3::Y,
            Self::Z => Vec3::Z,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    Clockwise,
    CounterClockwise,
}

impl Direction {
    /// Get the inverse of the direction.
    pub const fn inverse(self) -> Self {
        match self {
            Self::Clockwise => Self::CounterClockwise,
            Self::CounterClockwise => Self::Clockwise,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RotationMove {
    pub axis: RotationAxis,
    pub index: i32,
    pub direction: Direction,
    pub add_to_history: bool,
}

impl RotationMove {
    /// Get the inverse move.
    pub const fn inverse(self) -> Self {
        Self {
            axis: self.axis,
            index: self.index,
            direction: self.direction.inverse(),
            add_to_history: false,
        }
    }

    /// Get the rotation vector and angle for this move.
    pub fn get_rotation_info(self) -> (Vec3, f32) {
        let axis_vec = self.axis.vector();
        let angle = match self.direction {
            Direction::Clockwise => -std::f32::consts::FRAC_PI_2,
            Direction::CounterClockwise => std::f32::consts::FRAC_PI_2,
        };
        (axis_vec, angle)
    }

    /// Check if a cubie at the given coordinate is part of the rotating slice.
    pub const fn is_cubie_at_slice(self, coord: IVec3) -> bool {
        match self.axis {
            RotationAxis::X => coord.x == self.index,
            RotationAxis::Y => coord.y == self.index,
            RotationAxis::Z => coord.z == self.index,
        }
    }
}

/// Component attached to each colored face of a cubie.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CubieFace(pub Face);

/// Dynamic face mapping configuration for Front (F) and Down (D).
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FaceMapping {
    pub f_face: Face,
    pub d_face: Face,
    pub select_d_first: bool,
}

impl Default for FaceMapping {
    fn default() -> Self {
        Self {
            f_face: Face::Front,
            d_face: Face::Down,
            select_d_first: false,
        }
    }
}

impl FaceMapping {
    /// Return the static label ("U", "D", "L", "R", "F", "B") for a physical `Face` under current mapping.
    pub fn get_label_for_face(self, face: Face) -> &'static str {
        let normal = face.normal();
        let f_normal = self.f_face.normal();
        let d_normal = self.d_face.normal();
        let b_normal = -f_normal;
        let u_normal = -d_normal;
        let r_normal = f_normal.cross(d_normal);
        let l_normal = -r_normal;

        if normal.distance(f_normal) < 0.1 {
            "F"
        } else if normal.distance(d_normal) < 0.1 {
            "D"
        } else if normal.distance(b_normal) < 0.1 {
            "B"
        } else if normal.distance(u_normal) < 0.1 {
            "U"
        } else if normal.distance(r_normal) < 0.1 {
            "R"
        } else if normal.distance(l_normal) < 0.1 {
            "L"
        } else {
            "?"
        }
    }

    /// Return the logic `Face` corresponding to a physical `Face`.
    pub fn get_logic_face_for_physical(self, phys_face: Face) -> Face {
        let normal = phys_face.normal();
        let f_normal = self.f_face.normal();
        let d_normal = self.d_face.normal();
        let b_normal = -f_normal;
        let u_normal = -d_normal;
        let r_normal = f_normal.cross(d_normal);

        if normal.distance(f_normal) < 0.1 {
            Face::Front
        } else if normal.distance(d_normal) < 0.1 {
            Face::Down
        } else if normal.distance(b_normal) < 0.1 {
            Face::Back
        } else if normal.distance(u_normal) < 0.1 {
            Face::Up
        } else if normal.distance(r_normal) < 0.1 {
            Face::Right
        } else {
            Face::Left
        }
    }

    /// Return the character representation of a physical color under current mapping.
    pub fn get_char_for_physical_color(self, color: Face) -> char {
        let label = self.get_label_for_face(color);
        label.chars().next().unwrap_or('?')
    }

    /// Calculate the dynamic physical face, right vector, and down vector for a logical `Face`.
    pub fn get_face_config(self, logic_face: Face) -> (Face, Vec3, Vec3) {
        let f_normal = self.f_face.normal();
        let d_normal = self.d_face.normal();
        let r_normal = f_normal.cross(d_normal);

        let (normal, right_vec, down_vec) = match logic_face {
            Face::Up => (-d_normal, r_normal, f_normal),
            Face::Down => (d_normal, r_normal, -f_normal),
            Face::Front => (f_normal, r_normal, d_normal),
            Face::Back => (-f_normal, -r_normal, d_normal),
            Face::Right => (r_normal, r_normal.cross(d_normal), d_normal),
            Face::Left => (-r_normal, (-r_normal).cross(d_normal), d_normal),
        };

        let phys_face = Face::from_normal(normal).unwrap_or(logic_face);
        (phys_face, right_vec, down_vec)
    }

    /// Convert a logical move to a physical `RotationMove`.
    pub fn logic_move_to_physical_move(
        self,
        logic_face: Face,
        is_inverse: bool,
        size: i32,
    ) -> RotationMove {
        let (phys_face, _, _) = self.get_face_config(logic_face);
        let axis = match phys_face {
            Face::Right | Face::Left => RotationAxis::X,
            Face::Up | Face::Down => RotationAxis::Y,
            Face::Front | Face::Back => RotationAxis::Z,
        };

        let (index, base_dir) = match phys_face {
            Face::Right | Face::Up | Face::Front => (size - 1, Direction::Clockwise),
            Face::Left | Face::Down | Face::Back => (0, Direction::CounterClockwise),
        };

        let direction = if is_inverse {
            base_dir.inverse()
        } else {
            base_dir
        };

        RotationMove {
            axis,
            index,
            direction,
            add_to_history: true,
        }
    }

    /// Convert a physical `RotationMove` to its logical string representation.
    pub fn physical_move_to_logic_string(self, m: RotationMove, size: i32) -> String {
        let phys_face = match (m.axis, m.index) {
            (RotationAxis::X, idx) if idx == size - 1 => Face::Right,
            (RotationAxis::X, 0) => Face::Left,
            (RotationAxis::Y, idx) if idx == size - 1 => Face::Up,
            (RotationAxis::Y, 0) => Face::Down,
            (RotationAxis::Z, idx) if idx == size - 1 => Face::Front,
            (RotationAxis::Z, 0) => Face::Back,
            _ => {
                let axis_char = match m.axis {
                    RotationAxis::X => "X",
                    RotationAxis::Y => "Y",
                    RotationAxis::Z => "Z",
                };
                return if m.direction == Direction::Clockwise {
                    format!("{axis_char}{}", m.index)
                } else {
                    format!("{axis_char}{}'", m.index)
                };
            }
        };

        let logic_face = self.get_logic_face_for_physical(phys_face);
        let base = match logic_face {
            Face::Up => "U",
            Face::Down => "D",
            Face::Left => "L",
            Face::Right => "R",
            Face::Front => "F",
            Face::Back => "B",
        };

        let base_dir = match phys_face {
            Face::Right | Face::Up | Face::Front => Direction::Clockwise,
            Face::Left | Face::Down | Face::Back => Direction::CounterClockwise,
        };

        if m.direction == base_dir {
            base.to_string()
        } else {
            format!("{base}'")
        }
    }

    /// Calculate the physical rotation to match the current F/D mapping.
    pub fn get_rotation(self) -> Quat {
        let v_f = self.f_face.normal();
        let v_d = self.d_face.normal();
        let mat = Mat3::from_cols(v_f.cross(v_d), -v_d, v_f).transpose();
        Quat::from_mat3(&mat)
    }
}
