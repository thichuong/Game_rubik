use crate::rubik::components::Face;
use bevy::prelude::*;

/// Structure representing a segment of a 3D voxel-style letter
pub struct VoxelBar {
    pub offset: Vec3,
    pub size: Vec3,
}

/// Helper to create 3D voxel coordinates for letter U
fn get_voxel_u(t: f32) -> Vec<VoxelBar> {
    vec![
        VoxelBar {
            offset: Vec3::new(-0.08, 0.0, 0.0),
            size: Vec3::new(t, 0.20, t),
        }, // Left vertical
        VoxelBar {
            offset: Vec3::new(0.08, 0.0, 0.0),
            size: Vec3::new(t, 0.20, t),
        }, // Right vertical
        VoxelBar {
            offset: Vec3::new(0.0, -0.075, 0.0),
            size: Vec3::new(0.21, t, t),
        }, // Bottom horizontal
    ]
}

/// Helper to create 3D voxel coordinates for letter D
fn get_voxel_d(t: f32) -> Vec<VoxelBar> {
    vec![
        VoxelBar {
            offset: Vec3::new(-0.08, 0.0, 0.0),
            size: Vec3::new(t, 0.20, t),
        }, // Left vertical
        VoxelBar {
            offset: Vec3::new(-0.01, 0.075, 0.0),
            size: Vec3::new(0.14, t, t),
        }, // Top horizontal
        VoxelBar {
            offset: Vec3::new(-0.01, -0.075, 0.0),
            size: Vec3::new(0.14, t, t),
        }, // Bottom horizontal
        VoxelBar {
            offset: Vec3::new(0.06, 0.0, 0.0),
            size: Vec3::new(t, 0.15, t),
        }, // Right vertical
    ]
}

/// Helper to create 3D voxel coordinates for letter L
fn get_voxel_l(t: f32) -> Vec<VoxelBar> {
    vec![
        VoxelBar {
            offset: Vec3::new(-0.08, 0.0, 0.0),
            size: Vec3::new(t, 0.20, t),
        }, // Left vertical
        VoxelBar {
            offset: Vec3::new(0.0, -0.075, 0.0),
            size: Vec3::new(0.21, t, t),
        }, // Bottom horizontal
    ]
}

/// Helper to create 3D voxel coordinates for letter R
fn get_voxel_r(t: f32) -> Vec<VoxelBar> {
    vec![
        VoxelBar {
            offset: Vec3::new(-0.08, 0.0, 0.0),
            size: Vec3::new(t, 0.20, t),
        }, // Left vertical
        VoxelBar {
            offset: Vec3::new(-0.01, 0.075, 0.0),
            size: Vec3::new(0.14, t, t),
        }, // Top horizontal
        VoxelBar {
            offset: Vec3::new(-0.01, 0.0, 0.0),
            size: Vec3::new(0.14, t, t),
        }, // Middle horizontal
        VoxelBar {
            offset: Vec3::new(0.06, 0.038, 0.0),
            size: Vec3::new(t, 0.075, t),
        }, // Right vertical top
        VoxelBar {
            offset: Vec3::new(0.06, -0.038, 0.0),
            size: Vec3::new(t, 0.075, t),
        }, // Right vertical bottom
    ]
}

/// Helper to create 3D voxel coordinates for letter F
fn get_voxel_f(t: f32) -> Vec<VoxelBar> {
    vec![
        VoxelBar {
            offset: Vec3::new(-0.08, 0.0, 0.0),
            size: Vec3::new(t, 0.20, t),
        }, // Left vertical
        VoxelBar {
            offset: Vec3::new(0.0, 0.075, 0.0),
            size: Vec3::new(0.21, t, t),
        }, // Top horizontal
        VoxelBar {
            offset: Vec3::new(-0.015, 0.0, 0.0),
            size: Vec3::new(0.13, t, t),
        }, // Middle horizontal
    ]
}

/// Helper to create 3D voxel coordinates for letter B
fn get_voxel_b(t: f32) -> Vec<VoxelBar> {
    vec![
        VoxelBar {
            offset: Vec3::new(-0.08, 0.0, 0.0),
            size: Vec3::new(t, 0.20, t),
        }, // Left vertical
        VoxelBar {
            offset: Vec3::new(-0.01, 0.075, 0.0),
            size: Vec3::new(0.14, t, t),
        }, // Top horizontal
        VoxelBar {
            offset: Vec3::new(-0.01, 0.0, 0.0),
            size: Vec3::new(0.14, t, t),
        }, // Middle horizontal
        VoxelBar {
            offset: Vec3::new(-0.01, -0.075, 0.0),
            size: Vec3::new(0.14, t, t),
        }, // Bottom horizontal
        VoxelBar {
            offset: Vec3::new(0.06, 0.038, 0.0),
            size: Vec3::new(t, 0.075, t),
        }, // Right vertical top
        VoxelBar {
            offset: Vec3::new(0.06, -0.038, 0.0),
            size: Vec3::new(t, 0.075, t),
        }, // Right vertical bottom
    ]
}

/// Define the 3D cuboid layout for each face label letter
pub fn get_voxel_bars(label: &str) -> Vec<VoxelBar> {
    let t = 0.05; // Increased thickness of the bars to make the letters look bolder and more robust
    match label {
        "U" => get_voxel_u(t),
        "D" => get_voxel_d(t),
        "L" => get_voxel_l(t),
        "R" => get_voxel_r(t),
        "F" => get_voxel_f(t),
        "B" => get_voxel_b(t),
        _ => vec![],
    }
}

/// Helper to map a Face direction to its corresponding color representation
pub const fn get_face_color(face: Face) -> Srgba {
    match face {
        Face::Up => Srgba::WHITE,
        Face::Down => Srgba::new(1.0, 0.9, 0.0, 1.0),
        Face::Left => Srgba::new(1.0, 0.4, 0.0, 1.0),
        Face::Right => Srgba::new(0.9, 0.1, 0.1, 1.0),
        Face::Front => Srgba::new(0.1, 0.7, 0.1, 1.0),
        Face::Back => Srgba::new(0.1, 0.4, 0.9, 1.0),
    }
}
