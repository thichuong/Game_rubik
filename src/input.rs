use crate::components::*;
use bevy::ecs::observer::On;
use bevy::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DragState>()
            .add_systems(Update, handle_keyboard_input)
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
            side: Side::Right,
            direction,
        });
    }
    if keyboard_input.just_pressed(KeyCode::KeyL) {
        rotation_queue.0.push_back(RotationMove {
            side: Side::Left,
            direction,
        });
    }
    if keyboard_input.just_pressed(KeyCode::KeyU) {
        rotation_queue.0.push_back(RotationMove {
            side: Side::Up,
            direction,
        });
    }
    if keyboard_input.just_pressed(KeyCode::KeyD) {
        rotation_queue.0.push_back(RotationMove {
            side: Side::Down,
            direction,
        });
    }
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        rotation_queue.0.push_back(RotationMove {
            side: Side::Front,
            direction,
        });
    }
    if keyboard_input.just_pressed(KeyCode::KeyB) {
        rotation_queue.0.push_back(RotationMove {
            side: Side::Back,
            direction,
        });
    }
}

fn handle_drag_start(
    trigger: On<Pointer<DragStart>>,
    mut drag_state: ResMut<DragState>,
    face_query: Query<&CubieFace>,
) {
    let entity = trigger.entity;
    if let Ok(cubie_face) = face_query.get(entity) {
        if let Some(pos) = trigger.hit.position {
            drag_state.start_face = Some((entity, cubie_face.face, pos));
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
    if let Some((start_entity, face, start_pos)) = drag_state.start_face.take() {
        let (camera, camera_transform) = *camera_query;

        let location = trigger.pointer_location.position;
        let Ok(ray) = camera.viewport_to_world(camera_transform, location) else {
            return;
        };

        let normal = face.normal();
        let distance = ray
            .intersect_plane(
                start_pos,
                InfinitePlane3d::new(Dir3::new(normal).unwrap_or(Dir3::Y)),
            )
            .unwrap_or(0.0);
        let end_pos = ray.get_point(distance);

        let delta: Vec3 = end_pos - start_pos;
        if delta.length() > 0.4 {
            if let Ok((_, child_of)) = face_query.get(start_entity) {
                if let Ok(grid_coord) = cubie_query.get(child_of.0) {
                    if let Some(rm) = determine_move_robust(face, grid_coord.0, delta) {
                        rotation_queue.0.push_back(rm);
                    }
                }
            }
        }
    }
}

/// Robustly determine the rotation move based on face normal and drag vector
fn determine_move_robust(face: Face, coord: IVec3, delta: Vec3) -> Option<RotationMove> {
    let normal = face.normal();
    // Drag vector in the plane of the face
    let drag_vec = delta - delta.dot(normal) * normal;

    // The rotation axis should be perpendicular to both the face normal and the drag vector
    let raw_axis = normal.cross(drag_vec);

    // Quantize the axis to the closest world axis
    let axis = if raw_axis.x.abs() > raw_axis.y.abs() && raw_axis.x.abs() > raw_axis.z.abs() {
        Vec3::X * raw_axis.x.signum()
    } else if raw_axis.y.abs() > raw_axis.z.abs() {
        Vec3::Y * raw_axis.y.signum()
    } else {
        Vec3::Z * raw_axis.z.signum()
    };

    // Determine the side and direction
    // If axis is X, we are rotating a slice around X axis.
    // The slice is determined by coord.x
    let side = if axis.x.abs() > 0.5 {
        match coord.x {
            1 => Side::Right,
            -1 => Side::Left,
            _ => return None, // Middle slice (could be handled but keeping it simple)
        }
    } else if axis.y.abs() > 0.5 {
        match coord.y {
            1 => Side::Up,
            -1 => Side::Down,
            _ => return None,
        }
    } else {
        match coord.z {
            1 => Side::Front,
            -1 => Side::Back,
            _ => return None,
        }
    };

    // Direction logic:
    // If our quantized axis is in the same direction as the side's normal, it's Clockwise.
    // Otherwise, it's CounterClockwise.
    let side_normal = match side {
        Side::Right => Vec3::X,
        Side::Left => Vec3::NEG_X,
        Side::Up => Vec3::Y,
        Side::Down => Vec3::NEG_Y,
        Side::Front => Vec3::Z,
        Side::Back => Vec3::NEG_Z,
    };

    let direction = if axis.dot(side_normal) > 0.0 {
        Direction::CounterClockwise
    } else {
        Direction::Clockwise
    };

    Some(RotationMove { side, direction })
}
