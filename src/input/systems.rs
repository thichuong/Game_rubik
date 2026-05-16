use crate::input::resources::DragState;
use crate::rubik::components::{CubieFace, Direction, RotationAxis, RotationMove, RubikCube};
use crate::rubik::resources::{MoveHistory, RotationQueue};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub fn handle_mouse_input(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut drag_state: ResMut<DragState>,
    mut rotation_queue: ResMut<RotationQueue>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    cubie_faces: Query<(Entity, &CubieFace, &GlobalTransform)>,
    cube_query: Single<&GlobalTransform, With<RubikCube>>,
) {
    let window = window_query.single().expect("No primary window");
    let (camera, camera_transform) = camera_query.single().expect("No camera");

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else {
        return;
    };

    if mouse_button.just_pressed(MouseButton::Left) {
        let mut closest_hit = None;
        let mut min_dist = f32::MAX;

        for (entity, cubie_face, transform) in cubie_faces.iter() {
            let normal = transform.back();
            let center = transform.translation();

            let denom = ray.direction.dot(*normal);
            if denom.abs() > 1e-6 {
                let t = (center.dot(*normal) - ray.origin.dot(*normal)) / denom;
                if t > 0.0 && t < min_dist {
                    let hit_point = ray.origin + *ray.direction * t;
                    let local_hit = hit_point - center;
                    let right = transform.right();
                    let up = transform.up();

                    if local_hit.dot(*right).abs() <= 0.51 && local_hit.dot(*up).abs() <= 0.51 {
                        min_dist = t;
                        closest_hit = Some((entity, *normal, hit_point, cubie_face.0));
                    }
                }
            }
        }

        if let Some((entity, normal, hit_point, face)) = closest_hit {
            info!("Face pressed: {:?} at hit point {:?}", face, hit_point);
            drag_state.start_face = Some((entity, normal, hit_point));
        }
    }

    if mouse_button.just_released(MouseButton::Left) {
        if let Some((start_entity, start_normal, start_hit_point)) = drag_state.start_face {
            // Project current ray onto the plane of the starting face
            let denom = ray.direction.dot(start_normal);
            if denom.abs() > 1e-6 {
                let t = (start_hit_point.dot(start_normal) - ray.origin.dot(start_normal)) / denom;
                let end_hit_point = ray.origin + *ray.direction * t;
                let drag_vec = end_hit_point - start_hit_point;

                info!(
                    "Drag vector: {:?}, length {:.2}",
                    drag_vec,
                    drag_vec.length()
                );

                if drag_vec.length() > 0.3 {
                    let drag_dir = drag_vec.normalize();
                    let rotation_axis_vec = start_normal.cross(drag_dir);

                    let mut best_axis = RotationAxis::X;
                    let mut max_dot = 0.0;
                    let mut best_local_axis_in_world = Vec3::X;
                    let cube_transform = *cube_query;
                    for axis in [RotationAxis::X, RotationAxis::Y, RotationAxis::Z] {
                        let local_axis_in_world =
                            cube_transform.affine().transform_vector3(axis.vector());
                        let dot = rotation_axis_vec.dot(local_axis_in_world).abs();
                        if dot > max_dot {
                            max_dot = dot;
                            best_axis = axis;
                            best_local_axis_in_world = local_axis_in_world;
                        }
                    }

                    let sign = rotation_axis_vec.dot(best_local_axis_in_world).signum();
                    let direction = if sign > 0.0 {
                        Direction::CounterClockwise
                    } else {
                        Direction::Clockwise
                    };

                    #[allow(clippy::cast_possible_truncation)]
                    let index = {
                        // Use the center of the cubie we started on for the index
                        if let Ok((_ent, _face, transform)) = cubie_faces.get(start_entity) {
                            let cubie_pos = transform.translation();
                            let local_pos = cube_transform
                                .affine()
                                .inverse()
                                .transform_point3(cubie_pos);
                            match best_axis {
                                RotationAxis::X => local_pos.x.round() as i32,
                                RotationAxis::Y => local_pos.y.round() as i32,
                                RotationAxis::Z => local_pos.z.round() as i32,
                            }
                        } else {
                            0
                        }
                    };

                    rotation_queue.0.push_back(RotationMove {
                        axis: best_axis,
                        index,
                        direction,
                        add_to_history: true,
                    });
                }
            }
        }
        drag_state.start_face = None;
    }
}

pub fn handle_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut rotation_queue: ResMut<RotationQueue>,
    mut history: ResMut<MoveHistory>,
) {
    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::KeyZ) {
        if let Some(last_move) = history.done.pop() {
            let inverse_move = last_move.inverse();
            rotation_queue.0.push_back(inverse_move);
            history.undone.push(last_move);
        }
    }

    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::KeyY) {
        if let Some(last_undone) = history.undone.pop() {
            let mut redo_move = last_undone;
            redo_move.add_to_history = true;
            rotation_queue.0.push_back(redo_move);
        }
    }
}
