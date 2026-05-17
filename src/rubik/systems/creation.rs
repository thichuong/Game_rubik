use crate::rubik::components::{Cubie, CubieFace, Face, FaceLabel3d, RubikCube};
use crate::rubik::resources::RubikMaterials;
use bevy::prelude::*;

pub mod voxel;

// Small gap between cubies
pub const GAP: f32 = 1.02;

pub fn setup_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Load skin textures
    let carbon_tex = asset_server.load("textures/carbon.png");
    let geometric_tex = asset_server.load("textures/geometric.png");
    let floral_tex = asset_server.load("textures/floral.png");

    // Premium materials with a soft matte look to avoid glare
    let rubik_materials = RubikMaterials {
        white: materials.add(StandardMaterial {
            base_color: Color::Srgba(Srgba::WHITE),
            perceptual_roughness: 0.8,
            metallic: 0.0,
            ..default()
        }),
        yellow: materials.add(StandardMaterial {
            base_color: Color::Srgba(Srgba::new(1.0, 0.9, 0.0, 1.0)),
            perceptual_roughness: 0.8,
            metallic: 0.0,
            ..default()
        }),
        red: materials.add(StandardMaterial {
            base_color: Color::Srgba(Srgba::new(0.9, 0.1, 0.1, 1.0)),
            perceptual_roughness: 0.8,
            metallic: 0.0,
            ..default()
        }),
        orange: materials.add(StandardMaterial {
            base_color: Color::Srgba(Srgba::new(1.0, 0.4, 0.0, 1.0)),
            perceptual_roughness: 0.8,
            metallic: 0.0,
            ..default()
        }),
        green: materials.add(StandardMaterial {
            base_color: Color::Srgba(Srgba::new(0.1, 0.7, 0.1, 1.0)),
            perceptual_roughness: 0.8,
            metallic: 0.0,
            ..default()
        }),
        blue: materials.add(StandardMaterial {
            base_color: Color::Srgba(Srgba::new(0.1, 0.2, 0.9, 1.0)),
            perceptual_roughness: 0.8,
            metallic: 0.0,
            ..default()
        }),
        black: materials.add(StandardMaterial {
            base_color: Color::Srgba(Srgba::new(0.05, 0.05, 0.05, 1.0)),
            perceptual_roughness: 0.3,
            ..default()
        }),
        carbon_tex,
        geometric_tex,
        floral_tex,
    };
    commands.insert_resource(rubik_materials);
}

pub fn spawn_rubik_cube(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<RubikMaterials>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube_root = commands
        .spawn((
            RubikCube,
            Transform::IDENTITY,
            Visibility::default(),
            InheritedVisibility::default(),
        ))
        .id();

    let cubie_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let face_mesh = meshes.add(Cuboid::new(0.85, 0.85, 0.02));

    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                let grid_coord = IVec3::new(x, y, z);
                let position = grid_coord.as_vec3() * GAP;

                let cubie_id = commands
                    .spawn((
                        Cubie,
                        crate::rubik::components::GridCoord(grid_coord),
                        Mesh3d(cubie_mesh.clone()),
                        MeshMaterial3d(materials.black.clone()),
                        Transform::from_translation(position),
                    ))
                    .id();

                commands.entity(cube_root).add_child(cubie_id);

                // Add faces
                if x == 1 {
                    spawn_face(
                        &mut commands,
                        cubie_id,
                        Face::Right,
                        &face_mesh,
                        &materials.red,
                    );
                } else if x == -1 {
                    spawn_face(
                        &mut commands,
                        cubie_id,
                        Face::Left,
                        &face_mesh,
                        &materials.orange,
                    );
                }
                if y == 1 {
                    spawn_face(
                        &mut commands,
                        cubie_id,
                        Face::Up,
                        &face_mesh,
                        &materials.white,
                    );
                } else if y == -1 {
                    spawn_face(
                        &mut commands,
                        cubie_id,
                        Face::Down,
                        &face_mesh,
                        &materials.yellow,
                    );
                }
                if z == 1 {
                    spawn_face(
                        &mut commands,
                        cubie_id,
                        Face::Front,
                        &face_mesh,
                        &materials.green,
                    );
                } else if z == -1 {
                    spawn_face(
                        &mut commands,
                        cubie_id,
                        Face::Back,
                        &face_mesh,
                        &materials.blue,
                    );
                }
            }
        }
    }

    // Spawn elegant white axes lines and face label text (U, D, L, R, F, B) from the center of each face outwards
    spawn_face_axes(
        &mut commands,
        cube_root,
        &mut meshes,
        &mut standard_materials,
    );
}

fn spawn_face(
    commands: &mut Commands,
    parent: Entity,
    face: Face,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
) {
    let normal = face.normal();
    let translation = normal * 0.501;
    let rotation = Quat::from_rotation_arc(Vec3::Z, normal);
    let face_id = commands
        .spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(translation).with_rotation(rotation),
            CubieFace(face),
        ))
        .id();
    commands.entity(parent).add_child(face_id);
}

/// Spawn colored lines and 3D voxel-style letter models for each face of the Rubik's cube
fn spawn_face_axes(
    commands: &mut Commands,
    cube_root: Entity,
    meshes: &mut ResMut<Assets<Mesh>>,
    standard_materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let line_mesh = meshes.add(Cuboid::new(0.012, 0.012, 1.2)); // Slightly thinner line for a more elegant look

    let faces_info = [
        (Face::Up, "U"),
        (Face::Down, "D"),
        (Face::Left, "L"),
        (Face::Right, "R"),
        (Face::Front, "F"),
        (Face::Back, "B"),
    ];

    let half_cube_size = 1.5 * GAP;
    let line_length = 1.2;
    let line_center_dist = half_cube_size + line_length / 2.0;
    let label_dist = half_cube_size + line_length + 0.12;

    for (face, label) in faces_info {
        let normal = face.normal();
        let face_color = voxel::get_face_color(face);

        // Create a faded, semi-transparent material for the axis lines (hologram laser effect)
        let mut line_color = face_color;
        line_color.alpha = 0.25; // 25% opacity for elegant blend
        let line_material = standard_materials.add(StandardMaterial {
            base_color: Color::Srgba(line_color),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });

        // Create a solid unlit material for the voxel letter
        let letter_material = standard_materials.add(StandardMaterial {
            base_color: Color::Srgba(face_color),
            unlit: true,
            ..default()
        });

        // Calculate the line transform
        let line_translation = normal * line_center_dist;
        let line_rotation = Quat::from_rotation_arc(Vec3::Z, normal);

        let line_id = commands
            .spawn((
                Mesh3d(line_mesh.clone()),
                MeshMaterial3d(line_material),
                Transform::from_translation(line_translation).with_rotation(line_rotation),
            ))
            .id();

        commands.entity(cube_root).add_child(line_id);

        // Spawn a parent entity for the 3D voxel letter
        // It is spawned independently in world space (not a child of cube_root)
        // so that it can maintain its screen-aligned rotation (billboard)
        // while updating its position relative to the Rubik's cube.
        let label_parent_id = commands
            .spawn((
                Transform::IDENTITY,
                Visibility::default(),
                InheritedVisibility::default(),
                FaceLabel3d {
                    face,
                    dist: label_dist,
                },
            ))
            .id();

        // Spawn each individual segment/bar of the 3D letter
        let bars = voxel::get_voxel_bars(label);
        for bar in bars {
            let bar_mesh = meshes.add(Cuboid::new(bar.size.x, bar.size.y, bar.size.z));
            let bar_id = commands
                .spawn((
                    Mesh3d(bar_mesh),
                    MeshMaterial3d(letter_material.clone()),
                    Transform::from_translation(bar.offset),
                ))
                .id();

            commands.entity(label_parent_id).add_child(bar_id);
        }
    }
}
