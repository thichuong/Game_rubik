use crate::components::{
    Cubie, CubieFace, CurrentlyRotating, Face, GridCoord, MoveHistory, RotationMove, RotationQueue,
    RubikCube, RubikMaterials, TargetRotation,
};
use bevy::prelude::*;

pub struct RubikPlugin;

#[derive(Component)]
struct Pivot;

type CubieQueryData = (&'static mut Transform, &'static mut GridCoord);
type CubieQueryFilter = (With<Cubie>, Without<Pivot>);

const GAP: f32 = 1.02; // Small gap between cubies

impl Plugin for RubikPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RotationQueue>()
            .init_resource::<MoveHistory>()
            .add_systems(Startup, (setup_materials, spawn_rubik_cube).chain())
            .add_systems(Update, (handle_rotation_queue, animate_rotation));
    }
}

fn setup_materials(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    // Premium materials with HSL-inspired vibrant colors
    let rubik_materials = RubikMaterials {
        white: materials.add(StandardMaterial {
            base_color: Color::Srgba(Srgba::WHITE),
            perceptual_roughness: 0.1,
            metallic: 0.1,
            ..default()
        }),
        yellow: materials.add(StandardMaterial {
            base_color: Color::Srgba(Srgba::new(1.0, 0.9, 0.0, 1.0)),
            perceptual_roughness: 0.1,
            metallic: 0.1,
            ..default()
        }),
        red: materials.add(StandardMaterial {
            base_color: Color::Srgba(Srgba::new(0.9, 0.1, 0.1, 1.0)),
            perceptual_roughness: 0.1,
            metallic: 0.1,
            ..default()
        }),
        orange: materials.add(StandardMaterial {
            base_color: Color::Srgba(Srgba::new(1.0, 0.4, 0.0, 1.0)),
            perceptual_roughness: 0.1,
            metallic: 0.1,
            ..default()
        }),
        green: materials.add(StandardMaterial {
            base_color: Color::Srgba(Srgba::new(0.1, 0.7, 0.1, 1.0)),
            perceptual_roughness: 0.1,
            metallic: 0.1,
            ..default()
        }),
        blue: materials.add(StandardMaterial {
            base_color: Color::Srgba(Srgba::new(0.1, 0.2, 0.9, 1.0)),
            perceptual_roughness: 0.1,
            metallic: 0.1,
            ..default()
        }),
        black: materials.add(StandardMaterial {
            base_color: Color::Srgba(Srgba::new(0.05, 0.05, 0.05, 1.0)),
            perceptual_roughness: 0.3,
            ..default()
        }),
    };
    commands.insert_resource(rubik_materials);
}

fn spawn_rubik_cube(
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
    let face_mesh = meshes.add(Cuboid::new(0.85, 0.85, 0.02)); // Beveled look by making face smaller

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
    let translation = normal * 0.501; // Slightly outside the cubie
    let rotation = Quat::from_rotation_arc(Vec3::Z, normal);
    let face_id = commands
        .spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(translation).with_rotation(rotation),
            CubieFace,
        ))
        .id();
    commands.entity(parent).add_child(face_id);
}

fn handle_rotation_queue(
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

    if let Some(rotation_move) = queue.0.pop_front() {
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
    }
}

fn animate_rotation(
    mut commands: Commands,
    time: Res<Time>,
    current: Option<ResMut<CurrentlyRotating>>,
    pivot_query: Single<(Entity, &mut Transform, &TargetRotation), With<Pivot>>,
    mut cubie_query: Query<CubieQueryData, CubieQueryFilter>,
    cube_root: Single<Entity, With<RubikCube>>,
    mut history: ResMut<MoveHistory>,
) {
    let Some(mut current) = current else { return };
    let (pivot_entity, mut pivot_transform, target) = pivot_query.into_inner();
    let root_entity = *cube_root;

    current.timer.tick(time.delta());
    let progress = current.timer.fraction();

    // Ease Out Quad
    let eased_progress = (1.0 - progress).mul_add(-(1.0 - progress), 1.0);
    pivot_transform.rotation = Quat::IDENTITY.slerp(target.0, eased_progress);

    if current.timer.is_finished() {
        // Complete rotation
        for &cubie_entity in &current.cubies {
            if let Ok((mut transform, mut coord)) = cubie_query.get_mut(cubie_entity) {
                // Logic update
                coord.rotate(current.rotation_axis, current.angle);

                // SNAP: Reset transform relative to RubikCube
                // Position must be exactly GAP * GridCoord
                transform.translation = coord.0.as_vec3() * GAP;

                // Rotation must be exactly a multiple of 90 degrees
                // We apply the rotation step to the PREVIOUS rotation
                let rot_step = Quat::from_axis_angle(current.rotation_axis, current.angle);
                transform.rotation = (rot_step * transform.rotation).normalize();

                // Re-attach to root
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
