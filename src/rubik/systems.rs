use crate::events::ResetCameraEvent;
use crate::rubik::components::{
    Cubie, CubieFace, Face, GridCoord, Pivot, RotationMove, RubikCube, TargetRotation,
};
use crate::rubik::resources::{
    CurrentlyRotating, MoveHistory, RotationQueue, RubikMaterials, RubikSkin, SkinType,
};
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

pub const GAP: f32 = 1.02; // Small gap between cubies

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

pub fn update_skins(
    skin: Res<RubikSkin>,
    rubik_materials: Res<RubikMaterials>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !skin.is_changed() {
        return;
    }

    let texture = match skin.current {
        SkinType::Classic => None,
        SkinType::Carbon => Some(rubik_materials.carbon_tex.clone()),
        SkinType::Geometric => Some(rubik_materials.geometric_tex.clone()),
        SkinType::Floral => Some(rubik_materials.floral_tex.clone()),
    };

    let face_materials = [
        &rubik_materials.white,
        &rubik_materials.yellow,
        &rubik_materials.red,
        &rubik_materials.orange,
        &rubik_materials.green,
        &rubik_materials.blue,
    ];

    for handle in face_materials {
        if let Some(mat) = materials.get_mut(handle) {
            mat.base_color_texture.clone_from(&texture);
        }
    }
}

pub fn spawn_rubik_cube(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<RubikMaterials>,
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
                        GridCoord(grid_coord),
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

pub fn handle_rotation_queue(
    mut commands: Commands,
    mut queue: ResMut<RotationQueue>,
    current: Option<Res<CurrentlyRotating>>,
    cubies: Query<(Entity, &GridCoord), With<Cubie>>,
    cube_root: Single<Entity, With<RubikCube>>,
) {
    if current.is_some() {
        return;
    }
    let root_entity = *cube_root;

    while let Some(rotation_move) = queue.0.pop_front() {
        // Enforce blocking of center slice rotations (index 0) in core logic
        if rotation_move.index == 0 {
            continue;
        }

        let (axis_vec, angle) = rotation_move.get_rotation_info();

        let pivot_id = commands
            .spawn((
                Pivot,
                Transform::IDENTITY,
                Visibility::default(),
                InheritedVisibility::default(),
                TargetRotation(Quat::from_axis_angle(axis_vec, angle)),
            ))
            .id();

        commands.entity(root_entity).add_child(pivot_id);

        let mut rotating_cubies = Vec::new();
        for (entity, coord) in cubies.iter() {
            if rotation_move.is_cubie_at_slice(coord.0) {
                commands.entity(entity).insert(ChildOf(pivot_id));
                rotating_cubies.push(entity);
            }
        }

        commands.insert_resource(CurrentlyRotating {
            axis: rotation_move.axis,
            index: rotation_move.index,
            direction: rotation_move.direction,
            timer: Timer::from_seconds(0.25, TimerMode::Once),
            rotation_axis: axis_vec,
            angle,
            cubies: rotating_cubies,
            add_to_history: rotation_move.add_to_history,
        });
        return;
    }
}

pub type CubieQuery<'w, 's> =
    Query<'w, 's, (&'static mut Transform, &'static mut GridCoord), (With<Cubie>, Without<Pivot>)>;

pub fn animate_rotation(
    mut commands: Commands,
    time: Res<Time>,
    current: Option<ResMut<CurrentlyRotating>>,
    pivot_query: Single<(Entity, &mut Transform, &TargetRotation), With<Pivot>>,
    mut cubie_query: CubieQuery,
    cube_root: Single<Entity, With<RubikCube>>,
    mut history: ResMut<MoveHistory>,
) {
    let Some(mut current) = current else { return };
    let (pivot_entity, mut pivot_transform, target) = pivot_query.into_inner();
    let root_entity = *cube_root;

    current.timer.tick(time.delta());
    let progress = current.timer.fraction();

    let eased_progress = (1.0 - progress).mul_add(-(1.0 - progress), 1.0);
    pivot_transform.rotation = Quat::IDENTITY.slerp(target.0, eased_progress);

    if current.timer.is_finished() {
        for &cubie_entity in &current.cubies {
            if let Ok((mut transform, mut coord)) = cubie_query.get_mut(cubie_entity) {
                coord.rotate(current.rotation_axis, current.angle);
                transform.translation = coord.0.as_vec3() * GAP;

                let rot_step = Quat::from_axis_angle(current.rotation_axis, current.angle);
                transform.rotation = (rot_step * transform.rotation).normalize();

                commands.entity(cubie_entity).insert(ChildOf(root_entity));
            }
        }

        if current.add_to_history {
            history.done.push(RotationMove {
                axis: current.axis,
                index: current.index,
                direction: current.direction,
                add_to_history: false,
            });
            history.undone.clear();
        }

        commands.entity(pivot_entity).despawn();
        commands.remove_resource::<CurrentlyRotating>();
    }
}

/// System to handle whole-cube rotation via RMB (Free 360-degree rotation)
pub fn update_rubik_rotation(
    mouse_button: Res<ButtonInput<MouseButton>>,
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    cube_query: Single<&mut Transform, With<RubikCube>>,
    camera_query: Single<&Transform, (With<Camera>, Without<RubikCube>)>,
) {
    if mouse_button.pressed(MouseButton::Right) {
        let mut transform = cube_query.into_inner();
        let cam_transform = *camera_query;

        let delta_x = accumulated_mouse_motion.delta.x * 0.005;
        let delta_y = accumulated_mouse_motion.delta.y * 0.005;

        // Horizontal drag -> rotate around camera's up axis (screen vertical)
        let rot_y = Quat::from_axis_angle(*cam_transform.up(), delta_x);
        // Vertical drag -> rotate around camera's right axis (screen horizontal)
        let rot_x = Quat::from_axis_angle(*cam_transform.right(), delta_y);

        // Apply rotation in world space relative to camera perspective
        transform.rotation = rot_y * rot_x * transform.rotation;
    }
}

/// System to handle cube rotation reset
pub fn handle_cube_reset(
    mut events: MessageReader<ResetCameraEvent>,
    mut cube_transform: Single<&mut Transform, With<RubikCube>>,
) {
    for _ in events.read() {
        cube_transform.rotation = Quat::IDENTITY;
    }
}
