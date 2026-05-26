use crate::rubik::components::{Cubie, CubieFace, Face, FaceLabel3d, RubikCube};
use crate::rubik::resources::{FaceMapping, MoveHistory, RotationQueue, RubikMaterials, RubikSize};
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
    rubik_size: Res<RubikSize>,
    face_mapping: Res<FaceMapping>,
) {
    spawn_rubik_cube_internal(
        &mut commands,
        rubik_size.size,
        &mut meshes,
        &materials,
        &mut standard_materials,
        *face_mapping,
    );
}

#[allow(clippy::cast_precision_loss)]
pub fn spawn_rubik_cube_internal(
    commands: &mut Commands,
    size: i32,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &Res<RubikMaterials>,
    standard_materials: &mut ResMut<Assets<StandardMaterial>>,
    mapping: FaceMapping,
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

    let offset = (size as f32 - 1.0) / 2.0;
    let scale = 3.0 / size as f32;
    let current_gap = GAP * scale;

    for x in 0..size {
        for y in 0..size {
            for z in 0..size {
                // Skip rendering inner cubies for optimal performance
                if x > 0 && x < size - 1 && y > 0 && y < size - 1 && z > 0 && z < size - 1 {
                    continue;
                }

                let grid_coord = IVec3::new(x, y, z);
                let position = (grid_coord.as_vec3() - Vec3::splat(offset)) * current_gap;

                let cubie_id = commands
                    .spawn((
                        Cubie,
                        crate::rubik::components::GridCoord(grid_coord),
                        Mesh3d(cubie_mesh.clone()),
                        MeshMaterial3d(materials.black.clone()),
                        Transform::from_translation(position).with_scale(Vec3::splat(scale)),
                    ))
                    .id();

                commands.entity(cube_root).add_child(cubie_id);

                // Add faces
                if x == size - 1 {
                    spawn_face(commands, cubie_id, Face::Right, &face_mesh, &materials.red);
                } else if x == 0 {
                    spawn_face(
                        commands,
                        cubie_id,
                        Face::Left,
                        &face_mesh,
                        &materials.orange,
                    );
                }
                if y == size - 1 {
                    spawn_face(commands, cubie_id, Face::Up, &face_mesh, &materials.white);
                } else if y == 0 {
                    spawn_face(
                        commands,
                        cubie_id,
                        Face::Down,
                        &face_mesh,
                        &materials.yellow,
                    );
                }
                if z == size - 1 {
                    spawn_face(
                        commands,
                        cubie_id,
                        Face::Front,
                        &face_mesh,
                        &materials.green,
                    );
                } else if z == 0 {
                    spawn_face(commands, cubie_id, Face::Back, &face_mesh, &materials.blue);
                }
            }
        }
    }

    // Spawn elegant white axes lines and face label text (U, D, L, R, F, B) from the center of each face outwards
    spawn_face_axes(
        commands,
        cube_root,
        size,
        scale,
        meshes,
        standard_materials,
        mapping,
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

#[allow(clippy::cast_precision_loss)]
fn spawn_face_labels(
    commands: &mut Commands,
    size: i32,
    meshes: &mut ResMut<Assets<Mesh>>,
    standard_materials: &mut ResMut<Assets<StandardMaterial>>,
    mapping: FaceMapping,
) {
    let scale = 3.0 / size as f32;
    let current_gap = GAP * scale;
    let half_cube_size = (size as f32 / 2.0) * current_gap;
    let line_length = 1.2;
    let label_dist = half_cube_size + line_length + 0.12;

    let faces_info = [
        (Face::Up, mapping.get_label_for_face(Face::Up)),
        (Face::Down, mapping.get_label_for_face(Face::Down)),
        (Face::Left, mapping.get_label_for_face(Face::Left)),
        (Face::Right, mapping.get_label_for_face(Face::Right)),
        (Face::Front, mapping.get_label_for_face(Face::Front)),
        (Face::Back, mapping.get_label_for_face(Face::Back)),
    ];

    for (face, label) in faces_info {
        let face_color = voxel::get_face_color(face);

        let letter_material = standard_materials.add(StandardMaterial {
            base_color: Color::Srgba(face_color),
            unlit: true,
            ..default()
        });

        // Spawn a parent entity for the 3D voxel letter
        let label_parent_id = commands
            .spawn((
                Transform::from_scale(Vec3::ONE),
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

#[allow(clippy::cast_precision_loss)]
fn spawn_face_axes(
    commands: &mut Commands,
    cube_root: Entity,
    size: i32,
    scale: f32,
    meshes: &mut ResMut<Assets<Mesh>>,
    standard_materials: &mut ResMut<Assets<StandardMaterial>>,
    mapping: FaceMapping,
) {
    let current_gap = GAP * scale;
    let line_length = 1.2;
    let line_mesh = meshes.add(Cuboid::new(0.012, 0.012, line_length));

    let faces_info = [
        Face::Up,
        Face::Down,
        Face::Left,
        Face::Right,
        Face::Front,
        Face::Back,
    ];

    let half_cube_size = (size as f32 / 2.0) * current_gap;
    let line_center_dist = half_cube_size + line_length / 2.0;

    for face in faces_info {
        let normal = face.normal();
        let face_color = voxel::get_face_color(face);

        // Create a faded, semi-transparent material for the axis lines (hologram laser effect)
        let mut line_color = face_color;
        line_color.alpha = 0.25;
        let line_material = standard_materials.add(StandardMaterial {
            base_color: Color::Srgba(line_color),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
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
    }

    // Spawn billboarded 3D voxel labels
    spawn_face_labels(commands, size, meshes, standard_materials, mapping);
}

/// System to recreate Rubik when `RubikSize` changes, or rotate it & recreate labels when `FaceMapping` changes
pub fn handle_rubik_resize(
    mut commands: Commands,
    rubik_size: Res<RubikSize>,
    face_mapping: Res<FaceMapping>,
    cube_query: Query<(Entity, &mut Transform), With<RubikCube>>,
    label_query: Query<Entity, With<FaceLabel3d>>,
    mut queue: ResMut<RotationQueue>,
    mut history: ResMut<MoveHistory>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<RubikMaterials>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    let size_changed = rubik_size.is_changed() && !rubik_size.is_added();
    let mapping_changed = face_mapping.is_changed() && !face_mapping.is_added();

    if size_changed {
        // Despawn old RubikCube and all children (cubies, faces, lines)
        for (entity, _) in &cube_query {
            commands.entity(entity).despawn();
        }
        // Despawn old FaceLabels (they are spawned independently in world space)
        for entity in &label_query {
            commands.entity(entity).despawn();
        }

        // Reset the rotation queue and move history to prevent state conflicts
        queue.0.clear();
        history.done.clear();
        history.undone.clear();

        // Spawn new cube with the selected size
        spawn_rubik_cube_internal(
            &mut commands,
            rubik_size.size,
            &mut meshes,
            &materials,
            &mut standard_materials,
            *face_mapping,
        );
    } else if mapping_changed {
        // If ONLY face mapping changed, DO NOT recreate the cube (keep scrambled state).
        // And DO NOT automatically rotate the Rubik's cube here (keep physical colors in place for easy user tracking).
        // Just regenerate the 3D Face Labels to reflect the new letter configurations.
        for entity in &label_query {
            commands.entity(entity).despawn();
        }
        spawn_face_labels(
            &mut commands,
            rubik_size.size,
            &mut meshes,
            &mut standard_materials,
            *face_mapping,
        );
    }
}
