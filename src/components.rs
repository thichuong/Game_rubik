use bevy::prelude::*;

/// Marker for the entire Rubik's cube root entity
#[derive(Component)]
pub struct RubikCube;

/// Marker for a cubie entity
#[derive(Component)]
pub struct Cubie;

/// Logical coordinates of a cubie in the 3x3x3 grid (-1 to 1)
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridCoord(pub IVec3);

impl GridCoord {
    pub fn rotate(&mut self, axis: Vec3, angle: f32) {
        let rotation = Quat::from_axis_angle(axis, angle);
        let rotated = rotation * self.0.as_vec3();
        self.0 = rotated.round().as_ivec3();
    }
}

/// Target rotation for animation
#[derive(Component)]
pub struct TargetRotation(pub Quat);

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
}

/// Enum representing the 6 faces of a cube
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    Up,    // +Y (White)
    Down,  // -Y (Yellow)
    Right, // +X (Red)
    Left,  // -X (Orange)
    Front, // +Z (Green)
    Back,  // -Z (Blue)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationAxis {
    X,
    Y,
    Z,
}

impl RotationAxis {
    pub const fn vector(self) -> Vec3 {
        match self {
            Self::X => Vec3::X,
            Self::Y => Vec3::Y,
            Self::Z => Vec3::Z,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Clockwise,
    CounterClockwise,
}

impl Direction {
    pub const fn inverse(self) -> Self {
        match self {
            Self::Clockwise => Self::CounterClockwise,
            Self::CounterClockwise => Self::Clockwise,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RotationMove {
    pub axis: RotationAxis,
    pub index: i32, // -1, 0, or 1
    pub direction: Direction,
    pub add_to_history: bool,
}

#[derive(Resource, Default)]
pub struct RotationQueue(pub std::collections::VecDeque<RotationMove>);

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

impl RotationMove {
    pub const fn inverse(self) -> Self {
        Self {
            axis: self.axis,
            index: self.index,
            direction: self.direction.inverse(),
            add_to_history: false,
        }
    }

    pub fn get_rotation_info(self) -> (Vec3, f32) {
        let axis_vec = self.axis.vector();
        let angle = match self.direction {
            Direction::Clockwise => -std::f32::consts::FRAC_PI_2,
            Direction::CounterClockwise => std::f32::consts::FRAC_PI_2,
        };
        (axis_vec, angle)
    }

    pub const fn is_cubie_at_slice(self, coord: IVec3) -> bool {
        match self.axis {
            RotationAxis::X => coord.x == self.index,
            RotationAxis::Y => coord.y == self.index,
            RotationAxis::Z => coord.z == self.index,
        }
    }
}

impl Face {
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
}

/// Component attached to each colored face of a cubie
#[derive(Component)]
pub struct CubieFace;

/// Resource to track mouse drag for rotation
#[derive(Resource, Default)]
pub struct DragState {
    pub start_face: Option<(Entity, Vec3, Vec3)>,
}

/// Component for the orbiting camera
#[derive(Component)]
pub struct OrbitCamera {
    pub radius: f32,
    pub alpha: f32, // Horizontal angle (yaw)
    pub beta: f32,  // Vertical angle (pitch)
}
