use bevy::prelude::*;

/// Marker for the entire Rubik's cube root entity
#[derive(Component)]
pub struct RubikCube;

/// Marker for a cubie entity
#[derive(Component)]
pub struct Cubie;

/// Marker for the pivot used during rotation
#[derive(Component)]
pub struct Pivot;

/// Logical coordinates of a cubie in the `NxNxN` grid (0 to size - 1)
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridCoord(pub IVec3);

impl GridCoord {
    /// Update the logical coordinate based on a 90-degree rotation
    #[allow(clippy::cast_precision_loss)]
    pub fn rotate(&mut self, axis: Vec3, angle: f32, size: i32) {
        let offset = (size as f32 - 1.0) / 2.0;
        let rotation = Quat::from_axis_angle(axis, angle);
        let centered = self.0.as_vec3() - Vec3::splat(offset);
        let rotated = rotation * centered;
        let restored = rotated + Vec3::splat(offset);
        self.0 = restored.round().as_ivec3();
    }
}

/// Target rotation for animation
#[derive(Component)]
pub struct TargetRotation(pub Quat);

pub use rubik_solver::{CubieFace, Direction, Face, RotationAxis, RotationMove};

/// Component attached to the 3D voxel face labels to keep track of their target face and distance
#[derive(Component, Debug, Clone, Copy)]
pub struct FaceLabel3d {
    pub face: Face,
    pub dist: f32,
}
