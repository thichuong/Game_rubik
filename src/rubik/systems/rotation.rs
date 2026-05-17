use super::creation::GAP;
use crate::rubik::components::{Cubie, GridCoord, Pivot, RotationMove, RubikCube, TargetRotation};
use crate::rubik::resources::{CurrentlyRotating, MoveHistory, RotationQueue};
use bevy::prelude::*;

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
