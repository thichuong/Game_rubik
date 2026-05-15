use crate::components::{
    CubieFace, Direction, DragState, GridCoord, MoveHistory, RotationAxis, RotationMove,
    RotationQueue,
};
use bevy::ecs::observer::On;
use bevy::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DragState>()
            .add_systems(Update, (handle_keyboard_input, handle_undo_redo))
            .add_observer(handle_drag_start)
            .add_observer(handle_drag_end);
    }
}

fn handle_keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut rotation_queue: ResMut<RotationQueue>,
) {
    let direction = if keyboard_input.pressed(KeyCode::ShiftLeft)
        || keyboard_input.pressed(KeyCode::ShiftRight)
    {
        Direction::CounterClockwise
    } else {
        Direction::Clockwise
    };

    if keyboard_input.just_pressed(KeyCode::KeyR) {
        rotation_queue.0.push_back(RotationMove {
            axis: RotationAxis::X,
            index: 1,
            direction,
            add_to_history: true,
        });
    }
    if keyboard_input.just_pressed(KeyCode::KeyL) {
        rotation_queue.0.push_back(RotationMove {
            axis: RotationAxis::X,
            index: -1,
            direction,
            add_to_history: true,
        });
    }
    if keyboard_input.just_pressed(KeyCode::KeyU) {
        rotation_queue.0.push_back(RotationMove {
            axis: RotationAxis::Y,
            index: 1,
            direction,
            add_to_history: true,
        });
    }
    if keyboard_input.just_pressed(KeyCode::KeyD) {
        rotation_queue.0.push_back(RotationMove {
            axis: RotationAxis::Y,
            index: -1,
            direction,
            add_to_history: true,
        });
    }
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        rotation_queue.0.push_back(RotationMove {
            axis: RotationAxis::Z,
            index: 1,
            direction,
            add_to_history: true,
        });
    }
    if keyboard_input.just_pressed(KeyCode::KeyB) {
        rotation_queue.0.push_back(RotationMove {
            axis: RotationAxis::Z,
            index: -1,
            direction,
            add_to_history: true,
        });
    }
}

fn handle_drag_start(
    trigger: On<Pointer<DragStart>>,
    mut drag_state: ResMut<DragState>,
    face_query: Query<(&CubieFace, &GlobalTransform)>,
) {
    let entity = trigger.entity;
    if let Ok((_, transform)) = face_query.get(entity) {
        if let Some(pos) = trigger.hit.position {
            // In Bevy, the "back" of the face (+Z) is its world normal
            let normal = *transform.back();
            drag_state.start_face = Some((entity, normal, pos));
        }
    }
}

fn handle_drag_end(
    trigger: On<Pointer<DragEnd>>,
    mut drag_state: ResMut<DragState>,
    mut rotation_queue: ResMut<RotationQueue>,
    face_query: Query<(&CubieFace, &ChildOf)>,
    cubie_query: Query<&GridCoord>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
) {
    if let Some((start_entity, normal, start_pos)) = drag_state.start_face.take() {
        let (camera, camera_transform) = *camera_query;

        let location = trigger.pointer_location.position;
        let Ok(ray) = camera.viewport_to_world(camera_transform, location) else {
            return;
        };

        let distance = ray
            .intersect_plane(start_pos, InfinitePlane3d::new(normal))
            .unwrap_or(0.0);
        let end_pos = ray.get_point(distance);

        let delta: Vec3 = end_pos - start_pos;
        if delta.length() > 0.4 {
            if let Ok((_, child_of)) = face_query.get(start_entity) {
                if let Ok(grid_coord) = cubie_query.get(child_of.0) {
                    if let Some(rm) = determine_move_robust(normal, grid_coord.0, delta) {
                        rotation_queue.0.push_back(rm);
                    }
                }
            }
        }
    }
}

fn determine_move_robust(normal: Vec3, coord: IVec3, delta: Vec3) -> Option<RotationMove> {
    // Drag vector in the plane of the face (remove normal component)
    let drag_vec = delta - delta.dot(normal) * normal;

    if drag_vec.length() < 0.1 {
        return None;
    }

    // The rotation axis is perpendicular to both the face normal and the drag direction
    let raw_rotation_axis = normal.cross(drag_vec);

    // Find the dominant axis of rotation
    let (axis, index) = if raw_rotation_axis.x.abs() > raw_rotation_axis.y.abs()
        && raw_rotation_axis.x.abs() > raw_rotation_axis.z.abs()
    {
        (RotationAxis::X, coord.x)
    } else if raw_rotation_axis.y.abs() > raw_rotation_axis.z.abs() {
        (RotationAxis::Y, coord.y)
    } else {
        (RotationAxis::Z, coord.z)
    };

    // Determine direction:
    // If the raw rotation axis is in the same direction as the quantized axis,
    // we want a CounterClockwise rotation around that axis (positive angle).
    let axis_vec = axis.vector();
    let direction = if raw_rotation_axis.dot(axis_vec) > 0.0 {
        Direction::CounterClockwise
    } else {
        Direction::Clockwise
    };

    Some(RotationMove {
        axis,
        index,
        direction,
        add_to_history: true,
    })
}

fn handle_undo_redo(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut history: ResMut<MoveHistory>,
    mut rotation_queue: ResMut<RotationQueue>,
) {
    let ctrl = keyboard_input.pressed(KeyCode::ControlLeft)
        || keyboard_input.pressed(KeyCode::ControlRight);
    let shift =
        keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight);

    if ctrl && keyboard_input.just_pressed(KeyCode::KeyZ) {
        if shift {
            // Redo (Ctrl+Shift+Z)
            if let Some(m) = history.undone.pop() {
                rotation_queue.0.push_back(m);
                history.done.push(m);
            }
        } else {
            // Undo (Ctrl+Z)
            if let Some(m) = history.done.pop() {
                rotation_queue.0.push_back(m.inverse());
                history.undone.push(m);
            }
        }
    }

    if ctrl && keyboard_input.just_pressed(KeyCode::KeyY) {
        // Redo (Ctrl+Y)
        if let Some(m) = history.undone.pop() {
            rotation_queue.0.push_back(m);
            history.done.push(m);
        }
    }
}
