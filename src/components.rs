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
pub enum Side {
    Up,
    Down,
    Right,
    Left,
    Front,
    Back,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Clockwise,
    CounterClockwise,
}

pub struct RotationMove {
    pub side: Side,
    pub direction: Direction,
}

#[derive(Resource, Default)]
pub struct RotationQueue(pub std::collections::VecDeque<RotationMove>);

#[derive(Resource)]
pub struct CurrentlyRotating {
    pub side: Side,
    pub direction: Direction,
    pub timer: Timer,
    pub axis: Vec3,
    pub angle: f32,
    pub cubies: Vec<Entity>,
}

impl Side {
    pub fn get_rotation_info(&self, direction: Direction) -> (Vec3, f32) {
        let (axis, angle_base) = match self {
            Side::Right => (Vec3::X, 1.0),
            Side::Left => (Vec3::NEG_X, 1.0),
            Side::Up => (Vec3::Y, 1.0),
            Side::Down => (Vec3::NEG_Y, 1.0),
            Side::Front => (Vec3::Z, 1.0),
            Side::Back => (Vec3::NEG_Z, 1.0),
        };

        let angle = match direction {
            Direction::Clockwise => std::f32::consts::FRAC_PI_2 * angle_base,
            Direction::CounterClockwise => -std::f32::consts::FRAC_PI_2 * angle_base,
        };

        (axis, angle)
    }

    pub fn is_cubie_at_side(&self, coord: IVec3) -> bool {
        match self {
            Side::Right => coord.x == 1,
            Side::Left => coord.x == -1,
            Side::Up => coord.y == 1,
            Side::Down => coord.y == -1,
            Side::Front => coord.z == 1,
            Side::Back => coord.z == -1,
        }
    }
}

impl Face {
    pub fn normal(&self) -> Vec3 {
        match self {
            Face::Up => Vec3::Y,
            Face::Down => Vec3::NEG_Y,
            Face::Right => Vec3::X,
            Face::Left => Vec3::NEG_X,
            Face::Front => Vec3::Z,
            Face::Back => Vec3::NEG_Z,
        }
    }
}

/// Component attached to each colored face of a cubie
#[derive(Component)]
pub struct CubieFace {
    pub face: Face,
}

/// Resource to track mouse drag for rotation
#[derive(Resource, Default)]
pub struct DragState {
    pub start_face: Option<(Entity, Face, Vec3)>,
}

/// Component for the orbiting camera
#[derive(Component)]
pub struct OrbitCamera {
    pub radius: f32,
    pub alpha: f32, // Horizontal angle (yaw)
    pub beta: f32,  // Vertical angle (pitch)
}
