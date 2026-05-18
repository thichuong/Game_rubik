use crate::rubik::components::{Direction, Face, RotationAxis, RotationMove};
use bevy::prelude::*;
use std::collections::VecDeque;

/// Resource to hold materials for the Rubik's cube
#[derive(Resource)]
pub struct RubikMaterials {
    pub white: Handle<StandardMaterial>,
    pub yellow: Handle<StandardMaterial>,
    pub red: Handle<StandardMaterial>,
    pub orange: Handle<StandardMaterial>,
    pub green: Handle<StandardMaterial>,
    pub blue: Handle<StandardMaterial>,
    pub black: Handle<StandardMaterial>,
    // Skin textures
    pub carbon_tex: Handle<Image>,
    pub geometric_tex: Handle<Image>,
    pub floral_tex: Handle<Image>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SkinType {
    #[default]
    Classic,
    Carbon,
    Geometric,
    Floral,
}

#[derive(Resource, Default)]
pub struct RubikSkin {
    pub current: SkinType,
}

#[derive(Resource, Default)]
pub struct RotationQueue(pub VecDeque<RotationMove>);

#[derive(Resource, Default)]
pub struct MoveHistory {
    pub done: Vec<RotationMove>,
    pub undone: Vec<RotationMove>,
}

#[derive(Resource)]
pub struct CurrentlyRotating {
    pub axis: RotationAxis,
    pub index: i32,
    pub direction: Direction,
    pub timer: Timer,
    pub rotation_axis: Vec3, // Actual vector for Quat
    pub angle: f32,
    pub cubies: Vec<Entity>,
    pub add_to_history: bool,
}

/// Resource to hold the current size of the Rubik's cube (`NxNxN`)
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RubikSize {
    pub size: i32,
}

impl Default for RubikSize {
    fn default() -> Self {
        Self { size: 3 }
    }
}

/// Dynamic face mapping configuration for Front (F) and Down (D)
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
    /// Return the static label ("U", "D", "L", "R", "F", "B") for a physical `Face` under current mapping
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

    /// Return the logic `Face` corresponding to a physical `Face`
    #[allow(dead_code)]
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

    /// Return the character representation of a physical color under current mapping
    pub fn get_char_for_physical_color(self, color: Face) -> char {
        let label = self.get_label_for_face(color);
        label.chars().next().unwrap_or('?')
    }

    /// Calculate the dynamic physical face, right vector, and down vector for a logical `Face`
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

    /// Convert a logical move to a physical `RotationMove`
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

    /// Convert a physical `RotationMove` to its logical string representation
    #[allow(dead_code)]
    pub fn physical_move_to_logic_string(self, m: RotationMove, size: i32) -> String {
        let phys_face = match (m.axis, m.index) {
            (RotationAxis::X, idx) if idx == size - 1 => Face::Right,
            (RotationAxis::X, 0) => Face::Left,
            (RotationAxis::Y, idx) if idx == size - 1 => Face::Up,
            (RotationAxis::Y, 0) => Face::Down,
            (RotationAxis::Z, idx) if idx == size - 1 => Face::Front,
            (RotationAxis::Z, 0) => Face::Back,
            _ => return format!("{:?}{}", m.axis, m.index),
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

    /// Calculate the physical rotation to match the current F/D mapping
    pub fn get_rotation(self) -> Quat {
        let v_f = self.f_face.normal();
        let v_d = self.d_face.normal();
        let mat = Mat3::from_cols(v_f.cross(v_d), -v_d, v_f).transpose();
        Quat::from_mat3(&mat)
    }
}
